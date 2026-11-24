//! External fixed evaluator. Scenario labels and explicit rule stay here.
use atom_port_engine::{Chaos, Engine, Event, Mask, Order, Phase, PortOrchestrator};
use std::collections::{HashMap, HashSet, VecDeque};

struct Fixture {
    id: &'static str,
    expected: bool,
    events: Vec<Event>,
}
fn attempts(
    id: &'static str,
    expected: bool,
    count: usize,
    spacing: u64,
    identities: impl Fn(usize) -> (u64, u64, u16),
) -> Fixture {
    Fixture {
        id,
        expected,
        events: (0..count)
            .map(|i| {
                let (origin, destination, port) = identities(i);
                Event {
                    time_ns: i as u64 * spacing,
                    origin,
                    destination,
                    port,
                    phase: Phase::Attempt,
                }
            })
            .collect(),
    }
}
fn corpus() -> [Fixture; 8] {
    let mut pairs = Vec::new();
    for i in 0..16u64 {
        pairs.push(Event {
            time_ns: i * 10_000_000,
            origin: 7,
            destination: 11,
            port: 4000 + i as u16,
            phase: Phase::Attempt,
        });
        pairs.push(Event {
            time_ns: i * 10_000_000 + 1_000_000,
            origin: 7,
            destination: 11,
            port: 4000 + i as u16,
            phase: Phase::Complete,
        });
    }
    [
        attempts("P1", true, 16, 10_000_000, |i| (7, 11, 4000 + i as u16)),
        attempts("P2", true, 12, 10_000_000, |i| (7, 11, 4000 + i as u16)),
        attempts("P3", true, 24, 10_000_000, |i| (7, 11, 4000 + i as u16)),
        attempts("P4", true, 16, 10_000_000, |i| (7, 11, 4000 + 5 * i as u16)),
        attempts("C1", false, 24, 10_000_000, |_| (7, 11, 4000)),
        Fixture {
            id: "C2",
            expected: false,
            events: pairs,
        },
        attempts("C3", false, 16, 10_000_000, |i| {
            (7, 11 + i as u64, 4000 + i as u16)
        }),
        attempts("C4", false, 16, 5_000_000_000, |i| (7, 11, 4000 + i as u16)),
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
        ("classical-replacement", full, true),
        ("base-only", Mask::BASE_ONLY, false),
    ]
}
fn rule(events: &[Event]) -> Option<usize> {
    let mut pairs: HashMap<(u64, u64), VecDeque<(u64, u16)>> = HashMap::new();
    for (index, event) in events.iter().enumerate() {
        let window = pairs.entry((event.origin, event.destination)).or_default();
        while window
            .front()
            .is_some_and(|(time, _)| event.time_ns - time > 2_000_000_000)
        {
            window.pop_front();
        }
        match event.phase {
            Phase::Attempt => window.push_back((event.time_ns, event.port)),
            Phase::Complete => {
                if let Some(position) = window.iter().position(|(_, port)| *port == event.port) {
                    window.remove(position);
                }
            }
        }
        let distinct: HashSet<u16> = window.iter().map(|(_, port)| *port).collect();
        if distinct.len() >= 12 {
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
        engine = engine.with_classical_replacement();
    }
    let mut row = ResultRow {
        first: None,
        peak: 0.0,
        valid: true,
    };
    for (index, event) in events.iter().enumerate() {
        let value = PortOrchestrator::observe(&mut engine, *event).expect("frozen valid trace");
        if value.rupture && row.first.is_none() {
            row.first = Some(index + 1);
        }
        row.peak = row.peak.max(value.peak_energy);
        row.valid &= engine.invariants_hold()
            && value
                .destination_norm
                .is_none_or(|norm| (norm - 1.0).abs() < 1e-12)
            && value
                .port_norm
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
    for (name, order) in [
        ("A", Order::ReactBeforeHabituate),
        ("B", Order::HabituateBeforeReact),
    ] {
        for (arm, chaos) in [
            ("plain", Chaos::Plain),
            ("logistic-phase", Chaos::LogisticAtPhase),
            ("henon-reaction", Chaos::HenonAtReaction),
            ("both", Chaos::Both),
            ("logistic-reaction", Chaos::LogisticAtReaction),
            ("henon-phase", Chaos::HenonAtPhase),
        ] {
            for fixture in &fixtures {
                emit(
                    name,
                    arm,
                    fixture,
                    run(order, chaos, Mask::FULL, false, &fixture.events),
                );
            }
        }
        for (arm, mask, replacement) in controls() {
            for fixture in &fixtures {
                emit(
                    name,
                    arm,
                    fixture,
                    run(order, Chaos::Plain, mask, replacement, &fixture.events),
                );
            }
        }
        for fixture in &fixtures {
            let first = rule(&fixture.events);
            emit(
                name,
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
        assert_eq!(
            run(
                Order::ReactBeforeHabituate,
                Chaos::Plain,
                Mask::FULL,
                false,
                &fixture.events
            ),
            run(
                Order::ReactBeforeHabituate,
                Chaos::Plain,
                Mask::FULL,
                false,
                &fixture.events
            )
        );
    }
}
