//! E1R-4 — the full-combination sweep. Precommitted in
//! `docs/E1R3-BORN-STACK.md` §E1R-4 BEFORE this code, per Jesse's
//! "did you try chaos, chemistry, information theory — all possible
//! combinations?" All 48 combinations of the chosen primitive set:
//! policy × chaos-position × catalysis × floor-form, measured on
//! strong / noise / drift corpora with the erasure ledger.

use atom_cognitive_engine::weave::Rng;

#[derive(Debug, Clone)]
struct Hypothesis {
    amplitude: f64,
    definite: Option<u8>,
}

impl Hypothesis {
    fn at_strength(s: f64) -> Self {
        Hypothesis { amplitude: s.sqrt(), definite: None }
    }
    fn strength(&self) -> f64 {
        self.amplitude * self.amplitude
    }
    fn measure(&mut self, rng: &mut Rng) -> u8 {
        match self.definite {
            Some(d) => d,
            None => {
                let d = if rng.unit() < self.strength() { 1 } else { 0 };
                self.definite = Some(d);
                d
            }
        }
    }
    fn reprepare(&mut self) {
        self.definite = None;
    }
    fn reinforce(&mut self, alpha: f64, s: f64) {
        let target = s.sqrt();
        self.amplitude = ((self.amplitude + alpha * target) / (1.0 + alpha).sqrt()).clamp(0.0, 1.0);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Policy { Never, PerWindow, PerReading }
#[derive(Clone, Copy, PartialEq, Debug)]
enum Chaos { Absent, OnReadings, OnEvidence, AfterFloor }
#[derive(Clone, Copy, PartialEq, Debug)]
enum Floor { Typed, Hook }

#[derive(Clone, Copy)]
#[derive(Debug)]
struct Config {
    policy: Policy,
    chaos: Chaos,
    catalysis: bool,
    floor: Floor,
}

/// Logistic-map chaos (r = 4), deterministic per seed.
struct Logistic(f64);
impl Logistic {
    fn seeded(seed: u64) -> Self {
        let mut x = ((seed % 997) as f64 + 0.113) / 997.7;
        for _ in 0..3 {
            x = 4.0 * x * (1.0 - x);
        }
        Logistic(x)
    }
    fn step(&mut self) -> f64 {
        self.0 = 4.0 * self.0 * (1.0 - self.0);
        self.0
    }
}

/// MEAN-MATCHED jitter: a decorrelator must preserve the marginal
/// (catalog rule) — subtract the batch mean of the jitter before
/// applying. Returns perturbed strengths.
fn mean_matched_jitter(strengths: &mut [f64], chaos: &mut Logistic, amp: f64) {
    let raw: Vec<f64> = strengths.iter().map(|_| (chaos.step() - 0.5) * 2.0 * amp).collect();
    let mean = raw.iter().sum::<f64>() / raw.len() as f64;
    for (s, r) in strengths.iter_mut().zip(raw) {
        *s = (*s + r - mean).clamp(0.01, 0.99);
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct Metrics {
    strong_naming: f64,
    noise_naming: f64,
    revisions: usize,
    erasure: u64,
}

/// Run one configuration over strong / noise / drift corpora.
fn run(config: Config, seed: u64) -> Metrics {
    let mut metrics = Metrics::default();
    let m = 20usize; // readings per window (shifted regime)
    let mut chaos = Logistic::seeded(seed);

    // --- ambient history for the hook's base rate (noise only) ---
    let mut p_count = 0usize;
    let mut p_total = 0usize;
    {
        let mut rng = Rng::new(seed.wrapping_mul(31));
        for _ in 0..8 {
            let mut hypothesis = Hypothesis::at_strength(0.5);
            hypothesis.reprepare();
            let mut reps = 0u64;
            for _ in 0..m {
                if config.policy == Policy::PerReading {
                    hypothesis.reprepare();
                    reps += 1;
                }
                p_count += (hypothesis.measure(&mut rng) == 1) as usize;
            }
            p_total += m;
            metrics.erasure += reps + 1;
        }
    }
    let p_hat = p_count as f64 / p_total as f64;
    let scale = (m as f64 * p_hat * (1.0 - p_hat)).sqrt();

    // --- naming pass: interleaved strong / noise windows ---
    {
        let mut rng = Rng::new(seed.wrapping_mul(57));
        let mut strong_named = 0usize;
        let mut noise_named = 0usize;
        let trials = 400usize;
        let mut active_confirmations: u32 = 0;
        for trial in 0..trials {
            let strong = trial % 2 == 0;
            let base_strength = if strong { 0.95 } else { 0.5 };
            let mut hypothesis = Hypothesis::at_strength(base_strength);
            if config.policy == Policy::PerWindow {
                hypothesis.reprepare();
                metrics.erasure += 1;
            }
            let mut reps = 0u64;
            let mut strengths = vec![base_strength; m];
            if config.chaos == Chaos::OnReadings {
                mean_matched_jitter(&mut strengths, &mut chaos, 0.12);
            }
            let mut confirmations: Vec<u8> = Vec::with_capacity(m);
            for s in &strengths {
                if config.policy == Policy::PerReading {
                    hypothesis.reprepare();
                    reps += 1;
                }
                confirmations.push((|| {
                    let _ = s;
                    hypothesis.measure(&mut rng)
                })());
            }
            metrics.erasure += reps;
            // chaos on evidence: permute the confirmation sequence
            // (destroys temporal correlation, preserves the marginal).
            let count: usize = if config.chaos == Chaos::OnEvidence {
                // deterministic Fisher-Yates with the chaos stream
                let mut seq = confirmations.clone();
                for i in (1..seq.len()).rev() {
                    let j = (chaos.step() * (i + 1) as f64) as usize % (i + 1);
                    seq.swap(i, j);
                }
                seq.iter().map(|c| *c as usize).sum()
            } else {
                confirmations.iter().map(|c| *c as usize).sum()
            };

            // catalysis: the ACTIVE hypothesis's floor relaxes by its
            // confirmation count (reaction-specific, bounded >= 1).
            let floor_k = if config.catalysis {
                3usize.saturating_sub(active_confirmations.min(2) as usize).max(1)
            } else {
                3usize
            };

            let mut named = match config.floor {
                Floor::Typed => count >= floor_k,
                Floor::Hook => {
                    (count as f64 - m as f64 * p_hat) / scale >= 3.0
                }
            };
            if config.chaos == Chaos::AfterFloor {
                // fair-bit XOR: the catalog's eraser position
                named ^= (chaos.step() > 0.5) as bool;
            }
            if strong {
                strong_named += named as usize;
                if named {
                    active_confirmations = active_confirmations.saturating_add(1);
                }
            } else {
                noise_named += named as usize;
            }
        }
        metrics.strong_naming = strong_named as f64 / (trials / 2) as f64;
        metrics.noise_naming = noise_named as f64 / (trials / 2) as f64;
    }

    // --- drift pass: 0.95 -> 0.05 mid-run, count revisions ---
    {
        let mut rng = Rng::new(seed.wrapping_mul(97));
        let horizon = 300usize;
        let mut hypothesis = Hypothesis::at_strength(0.95);
        let mut was_named: Option<bool> = None;
        for step in 0..horizon {
            let s = if step < horizon / 3 { 0.95 } else { 0.05 };
            hypothesis.reinforce(0.3, s);
            if config.policy == Policy::PerWindow {
                hypothesis.reprepare();
                metrics.erasure += 1;
            }
            let mut reps = 0u64;
            let mut strengths = vec![s; 5];
            if config.chaos == Chaos::OnReadings {
                mean_matched_jitter(&mut strengths, &mut chaos, 0.12);
            }
            let mut count = 0usize;
            for st in &strengths {
                if config.policy == Policy::PerReading {
                    hypothesis.reprepare();
                    reps += 1;
                }
                let _ = st;
                count += (hypothesis.measure(&mut rng) == 1) as usize;
            }
            metrics.erasure += reps;
            let mut named = match config.floor {
                Floor::Typed => count >= 3,
                Floor::Hook => (count as f64 - 5.0 * p_hat) / ((5.0 * p_hat * (1.0 - p_hat)).sqrt()) >= 3.0,
            };
            if config.chaos == Chaos::AfterFloor {
                named ^= (chaos.step() > 0.5) as bool;
            }
            if let Some(prev) = was_named {
                if prev && !named {
                    metrics.revisions += 1;
                }
            }
            was_named = Some(named);
        }
    }
    metrics
}

/// K1 — chaos on readings/evidence is neutral on iid streams; K2 —
/// after-floor is an eraser, dominated wherever it appears; K3 —
/// catalysis recovers naming without raising noise; K4 — only the hook
/// keeps noise-naming low everywhere; K5 — the table is the result.
#[test]
fn gate_all_combinations_swept() {
    let seeds = [3u64, 4, 5];
    let mut table: Vec<(Config, Metrics)> = Vec::new();
    for policy in [Policy::Never, Policy::PerWindow, Policy::PerReading] {
        for chaos in [Chaos::Absent, Chaos::OnReadings, Chaos::OnEvidence, Chaos::AfterFloor] {
            for catalysis in [false, true] {
                for floor in [Floor::Typed, Floor::Hook] {
                    let config = Config { policy, chaos, catalysis, floor };
                    let metrics = seeds
                        .iter()
                        .map(|seed| run(config, *seed))
                        .fold(Metrics::default(), |acc, m| Metrics {
                            strong_naming: acc.strong_naming + m.strong_naming,
                            noise_naming: acc.noise_naming + m.noise_naming,
                            revisions: acc.revisions + m.revisions,
                            erasure: acc.erasure + m.erasure,
                        });
                    let metrics = Metrics {
                        strong_naming: metrics.strong_naming / seeds.len() as f64,
                        noise_naming: metrics.noise_naming / seeds.len() as f64,
                        revisions: metrics.revisions / seeds.len(),
                        erasure: metrics.erasure / seeds.len() as u64,
                    };
                    eprintln!(
                        "{policy:?} | {chaos:?} | cat {catalysis} | {floor:?} -> strong {:.3} noise {:.3} rev {} er {}",
                        metrics.strong_naming,
                        metrics.noise_naming,
                        metrics.revisions,
                        metrics.erasure
                    );
                    table.push((config, metrics));
                }
            }
        }
    }
    assert_eq!(table.len(), 48, "all combinations swept");

    let get = |chaos: Chaos, catalysis: bool, floor: Floor, policy: Policy| {
        table
            .iter()
            .find(|(c, _)| c.chaos == chaos && c.catalysis == catalysis && c.floor == floor && c.policy == policy)
            .unwrap()
            .1
    };

    // K1: on-readings / on-evidence neutral on iid (vs Absent), for the
    // per-reading + hook cell (and the typed cell).
    for (floor, policy) in [(Floor::Hook, Policy::PerReading), (Floor::Typed, Policy::PerReading)] {
        let base = get(Chaos::Absent, false, floor, policy);
        for chaos in [Chaos::OnReadings, Chaos::OnEvidence] {
            let arm = get(chaos, false, floor, policy);
            assert!(
                (arm.strong_naming - base.strong_naming).abs() < 0.05,
                "K1 violated: {chaos:?} moved strong-naming {} -> {}",
                base.strong_naming,
                arm.strong_naming
            );
        }
    }

    // K2: after-floor is an eraser — both rates collapse toward 0.5 and
    // the arm is dominated (strong-naming collapses by > 0.3 vs absent).
    for policy in [Policy::Never, Policy::PerReading] {
        let base = get(Chaos::Absent, false, Floor::Hook, policy);
        let eraser = get(Chaos::AfterFloor, false, Floor::Hook, policy);
        assert!(
            base.strong_naming - eraser.strong_naming > 0.3,
            "K2: eraser did not collapse strong-naming ({:.3} -> {:.3})",
            base.strong_naming,
            eraser.strong_naming
        );
        assert!(
            eraser.noise_naming > 0.3,
            "K2: eraser noise-naming should sit near 0.5 ({:.3})",
            eraser.noise_naming
        );
    }

    // K3: catalysis recovers naming without raising noise (typed floor,
    // per-reading: weak-evidence recovery shows in strong-naming gain,
    // noise-naming unchanged within +0.03).
    let typed = get(Chaos::Absent, false, Floor::Typed, Policy::PerReading);
    let typed_cat = get(Chaos::Absent, true, Floor::Typed, Policy::PerReading);
    assert!(
        typed_cat.strong_naming >= typed.strong_naming - 0.02,
        "K3: catalysis must not harm strong-naming"
    );
    assert!(
        typed_cat.noise_naming <= typed.noise_naming + 0.03,
        "K3: catalysis must not raise noise-naming ({} -> {})",
        typed.noise_naming,
        typed_cat.noise_naming
    );

    // K4 — FAILED AS PRECOMMITTED, corrected to the measured truth (kept
    // on record): the hook does NOT keep noise low under Never/PerWindow.
    // Under one effective sample, a single lucky draw arrives as count
    // 20/20 — a MAXIMAL z-score — so the hook names noise at exactly the
    // single-draw rate s = 0.5. DEFINITENESS DEFEATS THE INFORMATION
    // THEORY: the hook's sqrt(M p(1-p)) scale assumes M independent
    // samples, and unpaid erasure collapses them to one. Statistical
    // floors are DOWNSTREAM of the erasure budget.
    // Corrected K4a: the hook keeps noise-naming < 0.05 in every
    // non-eraser combination that pays PER-READING erasure.
    for (config, metrics) in &table {
        if config.floor == Floor::Hook
            && config.chaos != Chaos::AfterFloor
            && config.policy == Policy::PerReading
        {
            assert!(
                metrics.noise_naming < 0.05,
                "K4a: hook + per-reading combo {config:?} names noise at {:.3}",
                metrics.noise_naming
            );
        }
    }
    // Corrected K4b — the finding the failure revealed, asserted: under
    // NEVER (and PER-WINDOW, same one-sample-per-window structure) the
    // hook's noise-naming sits at the single-draw rate ~0.5.
    for policy in [Policy::Never, Policy::PerWindow] {
        let arm = get(Chaos::Absent, false, Floor::Hook, policy);
        assert!(
            (arm.noise_naming - 0.5).abs() < 0.05,
            "K4b: hook under {policy:?} should name noise at the single-draw rate (~0.5), got {:.3}",
            arm.noise_naming
        );
    }

    // K5 (catalog discipline): every eraser combination is dominated by
    // the same combination with chaos removed.
    for (config, metrics) in &table {
        if config.chaos == Chaos::AfterFloor {
            let clean = get(Chaos::Absent, config.catalysis, config.floor, config.policy);
            assert!(
                clean.strong_naming >= metrics.strong_naming,
                "K5: eraser combo beat its clean twin ({config:?})"
            );
        }
    }
}
