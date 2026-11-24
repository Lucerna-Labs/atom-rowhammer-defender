//! External frozen holdout. Labels and the programmed comparator stay here.
use atom_port_engine::{Chaos, Engine, Event, Mask, Order, Phase, PortOrchestrator};
use atom_port_principles::mix;
use std::collections::{HashMap, HashSet, VecDeque};

struct Fixture {
    seed: u64,
    name: &'static str,
    positive: bool,
    events: Vec<Event>,
}
fn attempts(
    seed: u64,
    name: &'static str,
    positive: bool,
    count: usize,
    spacing: u64,
    identities: impl Fn(usize) -> (u64, u64, u16),
) -> Fixture {
    Fixture {
        seed,
        name,
        positive,
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
fn corpus() -> Vec<Fixture> {
    (1..=16u64)
        .flat_map(|seed| {
            let base = 4000 + 37 * seed as u16;
            let destination = mix(1100 + seed);
            let mut pairs = Vec::new();
            for i in 0..16u64 {
                let port = base + i as u16;
                pairs.push(Event {
                    time_ns: i * 10_000_000,
                    origin: 7,
                    destination,
                    port,
                    phase: Phase::Attempt,
                });
                pairs.push(Event {
                    time_ns: i * 10_000_000 + 1_000_000,
                    origin: 7,
                    destination,
                    port,
                    phase: Phase::Complete,
                });
            }
            [
                attempts(seed, "contiguous", true, 16, 10_000_000, |i| {
                    (7, destination, base + i as u16)
                }),
                attempts(seed, "stride", true, 16, 10_000_000, |i| {
                    (7, destination, base + 5 * i as u16)
                }),
                attempts(seed, "permuted", true, 16, 10_000_000, |i| {
                    (7, destination, base + ((7 * i as u64 + seed) % 16) as u16)
                }),
                attempts(seed, "short", true, 12, 10_000_000, |i| {
                    (7, destination, base + i as u16)
                }),
                attempts(seed, "repeated", false, 24, 10_000_000, |_| {
                    (7, destination, base)
                }),
                Fixture {
                    seed,
                    name: "paired",
                    positive: false,
                    events: pairs,
                },
                attempts(seed, "destinations", false, 16, 10_000_000, |i| {
                    (7, mix(1100 + seed * 100 + i as u64 + 1), base + i as u16)
                }),
                attempts(seed, "slow", false, 16, 5_000_000_000, |i| {
                    (7, destination, base + i as u16)
                }),
            ]
        })
        .collect()
}
fn rule(events: &[Event]) -> Option<usize> {
    let mut windows: HashMap<(u64, u64), VecDeque<(u64, u16)>> = HashMap::new();
    for (index, event) in events.iter().enumerate() {
        let window = windows
            .entry((event.origin, event.destination))
            .or_default();
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
struct Row {
    first: Option<usize>,
    peak: f64,
    valid: bool,
}
fn run(chaos: Chaos, replacement: bool, events: &[Event]) -> Row {
    let mut engine = Engine::new(Order::ReactBeforeHabituate, chaos, Mask::FULL, 0);
    if replacement {
        engine = engine.with_classical_replacement();
    }
    let mut row = Row {
        first: None,
        peak: 0.0,
        valid: true,
    };
    for (index, event) in events.iter().enumerate() {
        let value =
            PortOrchestrator::observe(&mut engine, *event).expect("frozen valid generated event");
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
fn emit(fixture: &Fixture, arm: &str, row: Row) {
    println!(
        "{},{},{},{},{},{:.12},{}",
        fixture.seed,
        fixture.name,
        u8::from(fixture.positive),
        arm,
        row.first.unwrap_or(0),
        row.peak,
        u8::from(row.valid)
    );
}
fn main() {
    let fixtures = corpus();
    assert_eq!(fixtures.len(), 128);
    println!("seed,scenario,positive,arm,first_rupture,peak_energy,invariants");
    for fixture in &fixtures {
        for (arm, chaos, replacement) in [
            ("plain", Chaos::Plain, false),
            ("logistic-phase", Chaos::LogisticAtPhase, false),
            ("henon-reaction", Chaos::HenonAtReaction, false),
            ("both", Chaos::Both, false),
            ("logistic-reaction", Chaos::LogisticAtReaction, false),
            ("classical-replacement", Chaos::Plain, true),
        ] {
            let row = run(chaos, replacement, &fixture.events);
            if arm == "both" {
                assert_eq!(row, run(chaos, replacement, &fixture.events));
            }
            emit(fixture, arm, row);
        }
        let first = rule(&fixture.events);
        emit(
            fixture,
            "non-atom-rule",
            Row {
                first,
                peak: first.map_or(0.0, |_| 1.0),
                valid: true,
            },
        );
    }
}
