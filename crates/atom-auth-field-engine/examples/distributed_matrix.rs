//! External evaluator. Test labels and explicit count comparator stay here.
use atom_auth_field_engine::{Chaos, Engine, Event, FieldOrchestrator, Mask, Order};
use std::collections::{HashMap, HashSet, VecDeque};

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
    ids: impl Fn(usize) -> (u64, u64, bool),
) -> Fixture {
    Fixture {
        id,
        expected,
        events: (0..count)
            .map(|index| {
                let (origin, principal, failure) = ids(index);
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
        fixture("P1", true, 24, 100_000_000, |i| (i as u64 + 1, 1, true)),
        fixture("P2", true, 16, 100_000_000, |i| (i as u64 + 1, 1, true)),
        fixture("P3", true, 40, 50_000_000, |i| {
            ((i / 2) as u64 + 1, 1, true)
        }),
        fixture("P4", true, 32, 100_000_000, |i| {
            (17 * i as u64 + 3, 1, true)
        }),
        fixture("C1", false, 24, 100_000_000, |_| (7, 1, true)),
        fixture("C2", false, 24, 100_000_000, |i| {
            (i as u64 + 1, i as u64 + 1, true)
        }),
        fixture("C3", false, 24, 100_000_000, |i| (i as u64 + 1, 1, false)),
        fixture("C4", false, 24, 5_000_000_000, |i| (i as u64 + 1, 1, true)),
    ]
}
fn controls() -> [(&'static str, Mask, bool); 8] {
    let full = Mask::FULL;
    [
        (
            "without-quantum",
            Mask {
                quantum: false,
                ..full
            },
            false,
        ),
        (
            "without-information",
            Mask {
                information: false,
                ..full
            },
            false,
        ),
        (
            "without-chemistry",
            Mask {
                chemistry: false,
                ..full
            },
            false,
        ),
        (
            "without-cognition",
            Mask {
                cognition: false,
                ..full
            },
            false,
        ),
        (
            "without-mechanics",
            Mask {
                mechanics: false,
                ..full
            },
            false,
        ),
        (
            "without-physics",
            Mask {
                physics: false,
                ..full
            },
            false,
        ),
        ("uniform-replacement", full, true),
        ("base-only", Mask::BASE_ONLY, false),
    ]
}
fn rule(events: &[Event]) -> Option<usize> {
    let mut windows: HashMap<u64, VecDeque<(u64, u64)>> = HashMap::new();
    for (index, event) in events.iter().enumerate() {
        if !event.failure {
            continue;
        }
        let window = windows.entry(event.principal).or_default();
        while window
            .front()
            .is_some_and(|(t, _)| event.time_ns - t > 3_000_000_000)
        {
            window.pop_front();
        }
        window.push_back((event.time_ns, event.origin));
        let distinct: HashSet<u64> = window.iter().map(|(_, origin)| *origin).collect();
        if distinct.len() >= 16 {
            return Some(index + 1);
        }
    }
    None
}
#[derive(Clone, Debug, PartialEq)]
struct ResultRow {
    first: Option<usize>,
    peak: f64,
    valid: bool,
}
fn run(order: Order, chaos: Chaos, mask: Mask, replacement: bool, events: &[Event]) -> ResultRow {
    let mut engine = Engine::new(order, chaos, mask, 0);
    if replacement {
        engine = engine.with_uniform_replacement();
    }
    let mut row = ResultRow {
        first: None,
        peak: 0.0,
        valid: true,
    };
    for (index, event) in events.iter().enumerate() {
        let value = FieldOrchestrator::observe(&mut engine, *event).expect("frozen valid input");
        if value.rupture && row.first.is_none() {
            row.first = Some(index + 1);
        }
        row.peak = row.peak.max(value.peak_energy);
        row.valid &= engine.invariants_hold()
            && value
                .wave_norm
                .is_none_or(|norm| (norm - 1.0).abs() < 1e-12);
    }
    row
}
fn emit(order: &str, arm: &str, fixture: &Fixture, row: ResultRow) {
    println!(
        "{order},{arm},{},{},{},{:.12},{}",
        fixture.id,
        u8::from(fixture.expected),
        row.first.unwrap_or(0),
        row.peak,
        u8::from(row.valid)
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
            ("logistic-wave", Chaos::LogisticAtWave),
            ("henon-spring", Chaos::HenonAtSpring),
            ("both", Chaos::Both),
            ("logistic-spring", Chaos::LogisticAtSpring),
            ("henon-wave", Chaos::HenonAtWave),
        ] {
            for fixture in &fixtures {
                emit(
                    order_name,
                    name,
                    fixture,
                    run(order, chaos, Mask::FULL, false, &fixture.events),
                );
            }
        }
        for (name, mask, replacement) in controls() {
            for fixture in &fixtures {
                emit(
                    order_name,
                    name,
                    fixture,
                    run(order, Chaos::Plain, mask, replacement, &fixture.events),
                );
            }
        }
        for fixture in &fixtures {
            let first = rule(&fixture.events);
            emit(
                order_name,
                "non-atom-rule",
                fixture,
                ResultRow {
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
            false,
            &fixture.events,
        );
        let second = run(
            Order::AdsorbBeforeHabituate,
            Chaos::Plain,
            Mask::FULL,
            false,
            &fixture.events,
        );
        assert_eq!(first, second, "exact repeat for {}", fixture.id);
    }
}
