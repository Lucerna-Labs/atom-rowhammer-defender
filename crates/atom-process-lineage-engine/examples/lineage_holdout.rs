use std::collections::{HashMap, HashSet};

use atom_process_lineage_engine::{evaluate, Birth, Chaos, Mask, Order};

fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn preorder(parent: u64, out: &mut Vec<u64>) {
    for child in [2 * parent, 2 * parent + 1] {
        if child <= 32 {
            out.push(child);
            preorder(child, out);
        }
    }
}

fn stream(seed: u64, scenario: &str) -> Vec<Birth> {
    let id = |index: u64| mix((seed << 32) | index).max(1);
    let interval = if scenario == "slow_binary" {
        500_000_000
    } else {
        50_000 + 50_000 * seed
    };
    let mut indices = if scenario == "paired_trees" {
        (2..=31).collect::<Vec<u64>>()
    } else {
        (2..=32).collect::<Vec<u64>>()
    };
    if scenario == "binary_dfs" {
        indices.clear();
        preorder(1, &mut indices);
    }
    indices
        .iter()
        .enumerate()
        .map(|(position, index)| {
            let parent = match scenario {
                "binary_bfs" | "binary_dfs" | "slow_binary" => id(index / 2),
                "ternary_bfs" => id(1 + (index - 2) / 3),
                "paired_trees" if *index <= 16 => id(index / 2),
                "paired_trees" => {
                    let local = index - 15;
                    if local / 2 == 1 {
                        id(1000)
                    } else {
                        id(15 + local / 2)
                    }
                }
                "observed_star" if *index == 2 => id(1),
                "observed_star" => id(2),
                "two_hubs" if *index <= 3 => id(1),
                "two_hubs" => id(2 + index % 2),
                "chain" if *index == 2 => id(1),
                "chain" => id(index - 1),
                _ => unreachable!(),
            };
            Birth {
                time_ns: position as u64 * interval,
                pid: id(*index),
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
    println!("seed,scenario,births,peak,crossing,norm_error,baseline");
    let mut rows = 0;
    for seed in 1..=16_u64 {
        for scenario in [
            "binary_bfs",
            "binary_dfs",
            "ternary_bfs",
            "paired_trees",
            "observed_star",
            "two_hubs",
            "chain",
            "slow_binary",
        ] {
            let events = stream(seed, scenario);
            let unique = events.iter().map(|event| event.pid).collect::<HashSet<_>>();
            if unique.len() != events.len() || events.len() > 31 {
                return Err("invalid generated stream".into());
            }
            let result = evaluate(
                &events,
                Order::PhaseThenSite,
                Chaos::Plain,
                Mask::FULL,
                seed,
            )?;
            let repeat = evaluate(
                &events,
                Order::PhaseThenSite,
                Chaos::Plain,
                Mask::FULL,
                seed,
            )?;
            if result != repeat
                || result.births != events.len()
                || result.phase_norm_error > 1e-10
                || !result.peak_energy.is_finite()
                || !(0.0..=64.0).contains(&result.peak_energy)
            {
                return Err("determinism or state invariant failed".into());
            }
            println!(
                "{seed},{scenario},{},{:.12},{},{:.12},{}",
                events.len(),
                result.peak_energy,
                result.first_crossing.unwrap_or(0),
                result.phase_norm_error,
                graph_baseline(&events)
            );
            rows += 1;
        }
    }
    eprintln!("rows={rows} repeat=pass invariants=pass");
    Ok(())
}
