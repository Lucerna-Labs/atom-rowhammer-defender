//! External evaluator. Fixture labels and the explicit rule live here only.
use std::collections::{HashMap, HashSet, VecDeque};

use atom_auth_engine::{AuthOrchestrator, Chaos, Engine, Event, Mask, Order};

#[derive(Clone)]
struct Fixture {
    id: &'static str,
    expected: bool,
    events: Vec<Event>,
}

fn fixture(
    id: &'static str,
    expected: bool,
    count: usize,
    spacing: u64,
    identity: impl Fn(usize) -> (u64, u64, bool),
) -> Fixture {
    Fixture {
        id,
        expected,
        events: (0..count)
            .map(|index| {
                let (origin, principal, failure) = identity(index);
                Event {
                    time_ns: index as u64 * spacing,
                    origin,
                    principal,
                    failure,
                }
            })
            .collect(),
    }
}

fn corpus() -> [Fixture; 8] {
    [
        fixture("P1", true, 24, 100_000_000, |i| (7, i as u64 + 1, true)),
        fixture("P2", true, 16, 100_000_000, |i| (7, i as u64 + 1, true)),
        fixture("P3", true, 40, 50_000_000, |i| {
            (7, (i / 2) as u64 + 1, true)
        }),
        fixture("P4", true, 32, 100_000_000, |i| {
            (7, 17 * i as u64 + 3, true)
        }),
        fixture("C1", false, 24, 100_000_000, |_| (7, 1, true)),
        fixture("C2", false, 24, 100_000_000, |i| (7, i as u64 + 1, false)),
        fixture("C3", false, 24, 5_000_000_000, |i| (7, i as u64 + 1, true)),
        fixture("C4", false, 24, 100_000_000, |i| {
            (i as u64 + 1, i as u64 + 1, true)
        }),
    ]
}

fn masks() -> [(&'static str, Mask, Option<f64>); 8] {
    let full = Mask::FULL;
    [
        (
            "without-quantum",
            Mask {
                quantum: false,
                ..full
            },
            None,
        ),
        (
            "without-information",
            Mask {
                information: false,
                ..full
            },
            None,
        ),
        (
            "without-chemistry",
            Mask {
                chemistry: false,
                ..full
            },
            None,
        ),
        (
            "without-cognition",
            Mask {
                cognition: false,
                ..full
            },
            None,
        ),
        (
            "without-mechanics",
            Mask {
                mechanics: false,
                ..full
            },
            None,
        ),
        (
            "without-physics",
            Mask {
                physics: false,
                ..full
            },
            None,
        ),
        ("constant-overlap", full, Some(0.5)),
        ("base-only", Mask::BASE_ONLY, None),
    ]
}

fn rule(events: &[Event]) -> Option<usize> {
    let mut windows: HashMap<u64, VecDeque<(u64, u64)>> = HashMap::new();
    for (index, event) in events.iter().enumerate() {
        if !event.failure {
            continue;
        }
        let window = windows.entry(event.origin).or_default();
        while window
            .front()
            .is_some_and(|(time, _)| event.time_ns - time > 3_000_000_000)
        {
            window.pop_front();
        }
        window.push_back((event.time_ns, event.principal));
        let distinct: HashSet<u64> = window.iter().map(|(_, principal)| *principal).collect();
        if distinct.len() >= 16 {
            return Some(index + 1);
        }
    }
    None
}

#[derive(Clone, Debug, PartialEq)]
struct Outcome {
    first: Option<usize>,
    peak: f64,
    valid: bool,
}

fn run(
    order: Order,
    chaos: Chaos,
    mask: Mask,
    replacement: Option<f64>,
    events: &[Event],
) -> Outcome {
    let mut engine = Engine::new(order, chaos, mask, 0);
    if let Some(value) = replacement {
        engine = engine
            .with_replacement_overlap(value)
            .expect("valid frozen control");
    }
    let mut outcome = Outcome {
        first: None,
        peak: 0.0,
        valid: true,
    };
    for (index, event) in events.iter().enumerate() {
        let measured = AuthOrchestrator::observe(&mut engine, *event).expect("valid frozen trace");
        if measured.rupture && outcome.first.is_none() {
            outcome.first = Some(index + 1);
        }
        outcome.peak = outcome.peak.max(measured.peak_energy);
        outcome.valid &= engine.invariants_hold()
            && measured
                .quantum_norm
                .is_none_or(|norm| (norm - 1.0).abs() <= 1e-12);
    }
    outcome
}

fn emit(order: &str, arm: &str, fixture: &Fixture, outcome: &Outcome) {
    println!(
        "{order},{arm},{},{},{},{:.12},{}",
        fixture.id,
        u8::from(fixture.expected),
        outcome.first.map_or(0, |n| n),
        outcome.peak,
        u8::from(outcome.valid)
    );
}

fn main() {
    let fixtures = corpus();
    println!("order,arm,fixture,expected,first_rupture,peak_energy,invariants");
    for (order_name, order) in [
        ("A", Order::AdsorbBeforeHabituate),
        ("B", Order::HabituateBeforeAdsorb),
    ] {
        for (name, chaos) in [
            ("plain", Chaos::Plain),
            ("logistic-overlap", Chaos::LogisticAtOverlap),
            ("henon-force", Chaos::HenonAtForce),
            ("both", Chaos::Both),
            ("logistic-force", Chaos::LogisticAtForce),
            ("henon-overlap", Chaos::HenonAtOverlap),
        ] {
            for fixture in &fixtures {
                let outcome = run(order, chaos, Mask::FULL, None, &fixture.events);
                emit(order_name, name, fixture, &outcome);
            }
        }
        for (name, mask, replacement) in masks() {
            for fixture in &fixtures {
                let outcome = run(order, Chaos::Plain, mask, replacement, &fixture.events);
                emit(order_name, name, fixture, &outcome);
            }
        }
        for fixture in &fixtures {
            let first = rule(&fixture.events);
            emit(
                order_name,
                "non-atom-rule",
                fixture,
                &Outcome {
                    first,
                    peak: first.map_or(0.0, |_| 1.0),
                    valid: true,
                },
            );
        }
    }
    for fixture in &fixtures {
        let first = run(
            Order::AdsorbBeforeHabituate,
            Chaos::Plain,
            Mask::FULL,
            None,
            &fixture.events,
        );
        let second = run(
            Order::AdsorbBeforeHabituate,
            Chaos::Plain,
            Mask::FULL,
            None,
            &fixture.events,
        );
        assert_eq!(
            first, second,
            "exact deterministic replay for {}",
            fixture.id
        );
    }
}
