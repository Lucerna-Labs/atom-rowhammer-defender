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
    let second = base.wrapping_add(1_u64 << 32);
    let spacing = match case {
        "one_near" => 4_200,
        "two_fast" => 3_500,
        "one_slow" => 8_000,
        _ => 4_000,
    };
    (0..4096_u64)
        .map(|index| {
            let fingerprint = match case {
                "one_near" | "one_slow" => base,
                "two_fast" => {
                    if index % 2 == 0 {
                        base
                    } else {
                        second
                    }
                }
                "two_paired" => {
                    if index % 4 < 2 {
                        base
                    } else {
                        second
                    }
                }
                "three_one" => {
                    if index % 4 < 3 {
                        base
                    } else {
                        second
                    }
                }
                _ => unreachable!(),
            };
            Event {
                time_ns: index * spacing,
                fingerprint,
            }
        })
        .collect()
}

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
    println!("seed,case,peak,crossing,norm_error,baseline_peak,baseline_crossing");
    let mut rows = 0;
    for seed in 1001..=1064_u64 {
        for case in [
            "one_near",
            "two_fast",
            "two_paired",
            "three_one",
            "one_slow",
        ] {
            let events = stream(seed, case);
            let result = evaluate(
                &events,
                Order::PhaseThenChemistry,
                Chaos::Plain,
                Mask::FULL,
                seed,
            )?;
            let repeat = evaluate(
                &events,
                Order::PhaseThenChemistry,
                Chaos::Plain,
                Mask::FULL,
                seed,
            )?;
            if result != repeat
                || result.events != 4096
                || result.max_phase_norm_error > 1e-10
                || !result.peak_energy.is_finite()
                || !(0.0..=4096.0).contains(&result.peak_energy)
            {
                return Err("repeat or state invariant failed".into());
            }
            let (baseline_peak, baseline_crossing) = baseline(&events);
            println!(
                "{seed},{case},{:.12},{},{:.12},{:.12},{}",
                result.peak_energy,
                result.first_crossing.unwrap_or(0),
                result.max_phase_norm_error,
                baseline_peak,
                baseline_crossing.unwrap_or(0)
            );
            rows += 1;
        }
    }
    eprintln!("rows={rows} repeats=invariants=pass");
    Ok(())
}
