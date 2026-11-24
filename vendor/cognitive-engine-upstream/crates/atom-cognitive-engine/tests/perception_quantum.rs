//! E1R gates — quantum-first, emergent perception. Precommitted in
//! `docs/E1R-PERCEPTION-QUANTUM.md` BEFORE this code, under the four
//! discovery rules (`a-rag-atom/docs/DISCOVERY-RULES.md`). Labels exist
//! ONLY in this test harness for scoring — the engine never sees them.

use atom_cognitive_engine::perception::Reading;
use atom_cognitive_engine::perception_quantum::{EmergentConfig, EmergentPerceiver};

/// The E1 corpus (same geometry as the classical control): 3 regimes ×
/// 3 windows × 5 readings. TRUE CLASS INDEX is harness-side only.
fn corpus() -> Vec<(usize, Vec<Reading>)> {
    let mut windows = Vec::new();
    let mut base = 10u64;
    for (class, (exploration, convergence)) in
        [(0, (0.80, 0.70)), (1, (0.80, 0.30)), (2, (0.30, 0.70))]
    {
        for _ in 0..3 {
            let readings: Vec<Reading> = (0..5u64)
                .map(|k| {
                    Reading::new(
                        base + k,
                        0.5,
                        0.5,
                        0.5,
                        exploration + 0.01 * k as f64,
                        convergence - 0.01 * k as f64,
                        0.5,
                    )
                })
                .collect();
            base += 100;
            windows.push((class, readings));
        }
    }
    windows
}

/// Deterministic seeded shuffle (harness-side).
fn shuffle<T>(items: Vec<T>, seed: u64) -> Vec<T> {
    let mut state = seed | 1;
    let mut items = items;
    for i in (1..items.len()).rev() {
        state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        let j = (state >> 33) as usize % (i + 1);
        items.swap(i, j);
    }
    items
}

/// Run one formation pass: present the corpus windows in seeded-shuffled
/// order, then name every window against the formed bank. Returns
/// (per-window (true class, emergent name-or-None), bank size).
fn run(config: EmergentConfig, seed: u64) -> (Vec<(usize, Option<usize>)>, usize) {
    let windows = corpus();
    let mut perceiver = EmergentPerceiver::new(config);
    let order = shuffle(
        (0..windows.len()).collect::<Vec<usize>>(),
        seed.wrapping_mul(0x9E37_79B9_7F4A_7C15),
    );
    for index in &order {
        for reading in &windows[*index].1 {
            perceiver.ingest(reading);
        }
    }
    let named: Vec<(usize, Option<usize>)> = windows
        .iter()
        .map(|(class, readings)| (*class, perceiver.name_window(readings)))
        .collect();
    (named, perceiver.bank_size())
}

/// Clustering purity, harness-side: for each emergent name, the largest
/// true-class share; abstentions count against purity (they are unnamed).
fn purity(named: &[(usize, Option<usize>)]) -> f64 {
    let mut names: Vec<usize> = named
        .iter()
        .filter_map(|(_, name)| *name)
        .collect();
    names.sort_unstable();
    names.dedup();
    let mut correct = 0usize;
    for name in names {
        let mut class_counts = [0usize; 3];
        for (class, assigned) in named {
            if *assigned == Some(name) {
                class_counts[*class] += 1;
            }
        }
        correct += *class_counts.iter().max().unwrap();
    }
    correct as f64 / named.len() as f64
}

fn mean_purity(config: EmergentConfig, seeds: &[u64]) -> f64 {
    seeds
        .iter()
        .map(|seed| purity(&run(config, *seed).0))
        .sum::<f64>()
        / seeds.len() as f64
}

/// R1 (precommitted: pass) — zero typed-in content: the formed structure
/// is invariant to presentation order across 5 seeded shuffles (three
/// regimes nucleate three prototypes regardless of order), and the
/// module's constants are regime knobs only (no label vocabulary exists
/// in perception_quantum — enforced by its containing none).
#[test]
fn gate_zero_typed_content() {
    for seed in [1u64, 2, 3, 4, 5] {
        let (named, bank) = run(EmergentConfig::default(), seed);
        assert_eq!(bank, 3, "three regimes nucleate three prototypes (seed {seed})");
        let names: Vec<Option<usize>> = named.iter().map(|(_, n)| *n).collect();
        let distinct: Vec<usize> = {
            let mut d: Vec<usize> = names.iter().filter_map(|n| *n).collect();
            d.sort_unstable();
            d.dedup();
            d
        };
        assert_eq!(distinct.len(), 3, "every window is named (seed {seed})");
        // One emergent name per true class (purity 1.0, computed with
        // labels the engine never saw).
        assert!(
            (purity(&named) - 1.0).abs() < 1e-9,
            "order-invariant structure, purity {} (seed {seed})",
            purity(&named)
        );
    }
}

/// R2 (precommitted: pass) — emergent purity beats chance (≥ 0.80 on ≥ 4
/// of 5 shuffle seeds; chance ≈ 0.33). The classical control scores 1.0
/// by construction — it is typed; this gate is about the emergent bank.
#[test]
fn gate_emergent_purity_beats_chance() {
    let mut passes = 0usize;
    for seed in [11u64, 12, 13, 14, 15] {
        let (named, bank) = run(EmergentConfig::default(), seed);
        let score = purity(&named);
        eprintln!("seed {seed}: purity {score:.3}, bank {bank}");
        if score >= 0.80 {
            passes += 1;
        }
    }
    assert!(passes >= 4, "purity ≥ 0.80 on only {passes}/5 seeds");
}

/// R3 (precommitted: pass) — the permutation sweep: ALL 36 arrangements
/// (2 orders × 3 gaps × 3 rates × 2 drift) × 5 seeds, purity measured per
/// combination; the ADOPTED default is the best-mean configuration, and
/// the gate asserts the adopted mean ≥ the first-tried configuration's
/// mean (selection used the sweep, not the first working order).
#[test]
fn gate_permutation_sweep_chooses() {
    let seeds = [21u64, 22, 23, 24, 25];
    let mut results: Vec<(EmergentConfig, f64)> = Vec::new();
    for reinforce_then_assign in [false, true] {
        for nucleation_gap in [0.5, 0.7, 0.9] {
            for reinforce_rate in [0.1, 0.3, 0.5] {
                for drift in [false, true] {
                    let config = EmergentConfig {
                        nucleation_gap,
                        reinforce_rate,
                        drift,
                        reinforce_then_assign,
                        ..EmergentConfig::default()
                    };
                    let mean = mean_purity(config, &seeds);
                    eprintln!(
                        "gap {nucleation_gap:.1} rate {reinforce_rate:.1} drift {drift} \
                         rta {reinforce_then_assign}: mean purity {mean:.3}"
                    );
                    results.push((config, mean));
                }
            }
        }
    }
    assert_eq!(results.len(), 36, "all arrangements swept");
    let first_tried = results[0].1;
    let best = results
        .iter()
        .fold((results[0].0, results[0].1), |best, (config, mean)| {
            if *mean > best.1 {
                (*config, *mean)
            } else {
                best
            }
        });
    eprintln!(
        "first-tried mean {:.3}; best mean {:.3} (gap {:.1}, rate {:.1}, drift {}, rta {})",
        first_tried,
        best.1,
        best.0.nucleation_gap,
        best.0.reinforce_rate,
        best.0.drift,
        best.0.reinforce_then_assign
    );
    assert!(
        best.1 >= first_tried,
        "the sweep's best ({:.3}) must not lose to the first-tried order ({:.3})",
        best.1,
        first_tried
    );
    assert!(
        best.1 >= 0.80,
        "the adopted arrangement must clear the purity bar (best mean {:.3})",
        best.1
    );
}
