use std::collections::HashMap;

use atom_exact_row_site_engine::{evaluate, Chaos, Event, Mask, Order, CAPACITY};

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn stream(seed: u64, case: &str) -> Vec<Event> {
    let base = mix(seed);
    let spacing = if case == "slow_one" { 100_000 } else { 4_000 };
    (0..4096_u64)
        .map(|index| {
            let key = match case {
                "one" | "slow_one" => 0,
                "two" => index % 2,
                "four" => index % 4,
                "eight" => index % 8,
                "unique" => index,
                _ => unreachable!(),
            };
            Event {
                time_ns: index * spacing,
                fingerprint: base.wrapping_add(key << 32),
            }
        })
        .collect()
}

/// Independent ordinary exact-ID decayed count with the matched time constants.
fn baseline(events: &[Event]) -> (f64, Option<usize>) {
    let mut sites: HashMap<u64, (f64, u64)> = HashMap::new();
    let mut max_concentration = 0.0_f64;
    let mut heat = 0.0_f64;
    let mut peak = 0.0_f64;
    let mut first = None;
    let mut last_global = None;
    for (index, event) in events.iter().enumerate() {
        let dt = last_global.map_or(0, |last| event.time_ns - last);
        max_concentration *= (-(dt as f64) / 1_000_000.0).exp();
        let site = sites
            .entry(event.fingerprint)
            .or_insert((0.0, event.time_ns));
        site.0 = site.0 * (-((event.time_ns - site.1) as f64) / 1_000_000.0).exp() + 0.5;
        site.1 = event.time_ns;
        max_concentration = max_concentration.max(site.0);
        let alpha = 1.0 - (-(dt as f64) / 100_000.0).exp();
        heat += alpha * (max_concentration - heat);
        peak = peak.max(heat);
        if first.is_none() && heat >= CAPACITY {
            first = Some(index + 1);
        }
        last_global = Some(event.time_ns);
    }
    (peak, first)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("seed,case,order,chaos,mask,peak,crossing,norm_error,selected,baseline_peak,baseline_crossing");
    let mut rows = 0;
    for seed in 1..=64_u64 {
        for case in ["one", "two", "four", "eight", "unique", "slow_one"] {
            let events = stream(seed, case);
            let (base_peak, base_crossing) = baseline(&events);
            let arms = [
                (Order::PhaseThenChemistry, Chaos::Plain, Mask::FULL, "full"),
                (Order::ChemistryThenPhase, Chaos::Plain, Mask::FULL, "full"),
                (
                    Order::PhaseThenChemistry,
                    Chaos::Plain,
                    Mask {
                        quantum: false,
                        ..Mask::FULL
                    },
                    "no_quantum",
                ),
                (
                    Order::PhaseThenChemistry,
                    Chaos::Plain,
                    Mask {
                        chemistry: false,
                        ..Mask::FULL
                    },
                    "no_chemistry",
                ),
                (
                    Order::PhaseThenChemistry,
                    Chaos::Plain,
                    Mask {
                        physics: false,
                        ..Mask::FULL
                    },
                    "no_physics",
                ),
            ];
            for (order, chaos, mask, name) in arms.into_iter().chain((seed <= 3).then_some((
                Order::PhaseThenChemistry,
                Chaos::LogisticKick,
                Mask::FULL,
                "full",
            ))) {
                let result = evaluate(&events, order, chaos, mask, seed)?;
                let repeat = evaluate(&events, order, chaos, mask, seed)?;
                if result != repeat
                    || result.events != 4096
                    || result.max_phase_norm_error > 1e-10
                    || !result.peak_energy.is_finite()
                    || result.peak_energy < 0.0
                {
                    return Err("repeat or state invariant failed".into());
                }
                if name == "no_quantum"
                    && ((result.peak_energy - base_peak).abs() > 1e-9
                        || result.first_crossing != base_crossing)
                {
                    return Err("matched ordinary comparator diverged".into());
                }
                println!(
                    "{seed},{case},{order:?},{chaos:?},{name},{:.12},{},{:.12},{},{:.12},{}",
                    result.peak_energy,
                    result.first_crossing.unwrap_or(0),
                    result.max_phase_norm_error,
                    result.chosen_chaos_steps,
                    base_peak,
                    base_crossing.unwrap_or(0)
                );
                rows += 1;
            }
            if case == "one" {
                let benign_copy = events.clone();
                if evaluate(
                    &events,
                    Order::PhaseThenChemistry,
                    Chaos::Plain,
                    Mask::FULL,
                    seed,
                )? != evaluate(
                    &benign_copy,
                    Order::PhaseThenChemistry,
                    Chaos::Plain,
                    Mask::FULL,
                    seed,
                )? {
                    return Err("identical interpretation witness diverged".into());
                }
            }
        }
    }
    eprintln!("rows={rows} repeats=invariants=baseline-match=pass");
    Ok(())
}
