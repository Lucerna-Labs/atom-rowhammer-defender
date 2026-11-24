use std::collections::{HashMap, HashSet};

use atom_process_lineage_engine::{evaluate, Birth, Chaos, Mask, Order};

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn stream(seed: u64, scenario: &str) -> Vec<Birth> {
    let id = |index: u64| mix((seed << 32) | index).max(1);
    let interval = if scenario == "slow_tree" {
        500_000_000
    } else {
        100_000
    };
    (2..=32_u64)
        .map(|index| {
            let parent = match scenario {
                "fast_tree" | "slow_tree" => id(index / 2),
                "star" => id(1),
                "chain" => id(index - 1),
                "forest" => id(100 + index),
                _ => unreachable!(),
            };
            Birth {
                time_ns: (index - 2) * interval,
                pid: id(index),
                ppid: parent,
            }
        })
        .collect()
}

fn graph_baseline(stream: &[Birth]) -> bool {
    let births = stream.iter().map(|event| event.pid).collect::<HashSet<_>>();
    let mut children: HashMap<u64, Vec<u64>> = HashMap::new();
    for event in stream {
        if births.contains(&event.ppid) {
            children.entry(event.ppid).or_default().push(event.time_ns);
        }
    }
    children
        .values()
        .filter(|times| {
            times.iter().any(|start| {
                times
                    .iter()
                    .filter(|end| **end >= *start && **end - *start <= 10_000_000)
                    .count()
                    >= 2
            })
        })
        .count()
        >= 4
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("seed,scenario,order,chaos,mask,peak,crossing,norm_error,chaos_steps,baseline");
    let mut rows = 0usize;
    for seed in 1..=16_u64 {
        for scenario in ["fast_tree", "star", "chain", "forest", "slow_tree"] {
            let events = stream(seed, scenario);
            let unique = events.iter().map(|event| event.pid).collect::<HashSet<_>>();
            if unique.len() != 31
                || events
                    .iter()
                    .any(|event| unique.contains(&event.ppid) && event.pid == event.ppid)
            {
                return Err("invalid generated identities".into());
            }
            for order in [Order::PhaseThenSite, Order::SiteThenPhase] {
                for chaos in [Chaos::Plain, Chaos::LogisticPhase] {
                    for (mask_name, mask) in [
                        ("full", Mask::FULL),
                        (
                            "no_quantum",
                            Mask {
                                quantum: false,
                                ..Mask::FULL
                            },
                        ),
                        (
                            "no_biology",
                            Mask {
                                biology: false,
                                ..Mask::FULL
                            },
                        ),
                        (
                            "no_chemistry",
                            Mask {
                                chemistry: false,
                                ..Mask::FULL
                            },
                        ),
                        (
                            "no_physics",
                            Mask {
                                physics: false,
                                ..Mask::FULL
                            },
                        ),
                    ] {
                        if mask_name != "full" && chaos != Chaos::Plain {
                            continue;
                        }
                        let result = evaluate(&events, order, chaos, mask, seed)?;
                        let repeat = evaluate(&events, order, chaos, mask, seed)?;
                        if result != repeat
                            || result.births != 31
                            || result.phase_norm_error > 1e-10
                            || !result.peak_energy.is_finite()
                            || result.peak_energy < 0.0
                        {
                            return Err("determinism or state invariant failed".into());
                        }
                        println!("{seed},{scenario},{order:?},{chaos:?},{mask_name},{:.12},{},{:.12},{},{}",
                            result.peak_energy,
                            result.first_crossing.map_or(0, |x| x),
                            result.phase_norm_error,
                            result.chosen_chaos_steps,
                            graph_baseline(&events));
                        rows += 1;
                    }
                }
            }
            if scenario == "fast_tree" {
                let interpreted_benign = events.clone();
                let attack_output = evaluate(
                    &events,
                    Order::PhaseThenSite,
                    Chaos::Plain,
                    Mask::FULL,
                    seed,
                )?;
                let benign_output = evaluate(
                    &interpreted_benign,
                    Order::PhaseThenSite,
                    Chaos::Plain,
                    Mask::FULL,
                    seed,
                )?;
                if attack_output != benign_output {
                    return Err("identical stream interpretation changed output".into());
                }
            }
        }
    }
    eprintln!("rows={rows} invariant=pass identical-interpretation=pass");
    Ok(())
}
