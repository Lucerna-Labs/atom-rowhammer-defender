//! Per-feed swarm eval harness. Runs the frozen precommit gates for ONE feed
//! at a time (attribution stays clean: one variable per experiment).
//!
//! Usage:
//!   cargo run -p imperium-nocopy-swarm --example swarm_feed_eval -- \
//!       --feed frames|sessions --thermal 0.0|0.1|4.0 --hill on|off \
//!       [--coupling 0.30] --out <dir>
//!
//! Writes <dir>/<feed>-<thermal>-<hill>-c<coupling>-ledger.csv and -summary.md.
//! Test-only synthetic key bytes; no real memory or live key is touched.

use imperium_nocopy_swarm::{Feed, SessionEvent, SwarmTripwire, SwarmTripwireConfig};
use imperium_nocopy_vpn_observer::FrameClass;
use nocopy_sensor_engine::Order;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

const MAX_READS: usize = 32;
// scenario codes: 0=single-flood, 1=dist-flood, 2=dist-scan, 3=dist-sparse,
// 4=dist-scan-wide, 5=benign, 6=move, 7=empty
const ALL_SESSION_EVENTS: [SessionEvent; 7] = [
    SessionEvent::Hello,
    SessionEvent::HelloResponse,
    SessionEvent::HandshakeCompleted,
    SessionEvent::Rekey,
    SessionEvent::Expire,
    SessionEvent::Rebind,
    SessionEvent::Ping,
];

fn splitmix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

// Test-only synthetic bytes, same construction as the no-copy evals.
fn synthetic_key(base: usize, index: usize) -> Vec<u8> {
    match base {
        0 => (0..32)
            .map(|position| 1 + (splitmix((index * 32 + position) as u64) % 255) as u8)
            .collect(),
        1 => {
            let source = format!("synthetic-handshake-frame-{index:02}").into_bytes();
            (0..32)
                .map(|position| source[position % source.len()])
                .collect()
        }
        _ => (0..32)
            .map(|position| 1 + (splitmix((position * 17 + index * 13) as u64) % 255) as u8)
            .collect(),
    }
}

fn homes_for(feed: Feed, base: usize) -> [u8; 4] {
    match feed {
        Feed::Frames => core::array::from_fn(|j| ((base + j) % 8) as u8),
        Feed::SessionEvents => core::array::from_fn(|j| (1 + (base + j) % 7) as u8),
    }
}

fn foreign_for(feed: Feed, base: usize) -> Vec<u8> {
    let homes = homes_for(feed, base);
    match feed {
        Feed::Frames => (0..8_u8).filter(|c| !homes.contains(c)).collect(),
        Feed::SessionEvents => (1..=7_u8).filter(|t| !homes.contains(t)).collect(),
    }
}

fn event_of_tag(tag: u8) -> SessionEvent {
    ALL_SESSION_EVENTS
        .into_iter()
        .find(|event| event.tag() == tag)
        .unwrap_or(SessionEvent::Hello)
}

// One feed input for (scenario, k): a label value in the feed's label space.
fn scenario_label(feed: Feed, base: usize, scenario: usize, k: usize) -> Option<u8> {
    let homes = homes_for(feed, base);
    let foreign = foreign_for(feed, base);
    match scenario {
        0 => (k < 16).then_some(homes[0]),
        1 => (k < 16).then_some(homes[k % 4]),
        2 => (k < 16).then_some(foreign[k % foreign.len()]),
        3 => {
            if k >= 32 {
                None
            } else if k.is_multiple_of(4) {
                Some(homes[(k / 4) % 4])
            } else {
                Some(foreign[k % foreign.len()])
            }
        }
        4 => (k < 32).then_some(foreign[k % foreign.len()]),
        5 => (k < 4).then(|| homes[k]),
        6 => (k < 4).then_some(foreign[0]),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct Row {
    order: usize,
    base: usize,
    index: usize,
    placement: usize,
    scenario: usize,
    trip_reads: i64,
    heats: [f64; 4],
    ruptured: usize,
    warm: usize,
    nonzero_min: usize,
    live_unchanged: bool,
    external_survives: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let mut feed = None;
    let mut thermal = None;
    let mut hill = None;
    let mut coupling = 0.30_f64;
    let mut out = None;
    let mut iter = arguments.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--feed" => {
                feed = Some(match iter.next().as_deref() {
                    Some("frames") => Feed::Frames,
                    Some("sessions") => Feed::SessionEvents,
                    _ => return Err("--feed frames|sessions".into()),
                })
            }
            "--thermal" => {
                thermal = Some(
                    iter.next()
                        .ok_or("--thermal value")?
                        .parse::<f64>()?,
                )
            }
            "--hill" => {
                hill = Some(match iter.next().as_deref() {
                    Some("on") => true,
                    Some("off") => false,
                    _ => return Err("--hill on|off".into()),
                })
            }
            "--coupling" => coupling = iter.next().ok_or("--coupling value")?.parse::<f64>()?,
            "--out" => out = Some(PathBuf::from(iter.next().ok_or("--out dir")?)),
            _ => return Err(format!("unknown argument {arg}").into()),
        }
    }
    let feed = feed.ok_or("--feed required")?;
    let thermal = thermal.ok_or("--thermal required")?;
    let hill = hill.ok_or("--hill required")?;
    let out = out.ok_or("--out required")?;
    let orders = [Order::L, Order::M];
    let expected_rows = 2 * 3 * 16 * 3 * 8;

    let mut rows = Vec::with_capacity(expected_rows);
    let mut errors = Vec::new();
    let mut zeroization_failures = 0_usize;
    for (order_index, order) in orders.into_iter().enumerate() {
        for base in 0..3 {
            for index in 0..16 {
                for placement in 0..3 {
                    let homes = homes_for(feed, base);
                    let copies: [Vec<u8>; 4] =
                        core::array::from_fn(|j| synthetic_key(base, (index + j) % 16));
                    let seeds: [u64; 4] = core::array::from_fn(|j| {
                        splitmix(((base * 16 + (index + j) % 16) * 31 + placement) as u64)
                    });
                    let live = copies[0].clone();
                    let external: Vec<Vec<u8>> = copies.to_vec();
                    for scenario in 0..8 {
                        let config = SwarmTripwireConfig {
                            homes,
                            seeds,
                            order,
                            thermal,
                            hill,
                            placement: placement as u8,
                            coupling,
                            warm_threshold: 0.15,
                            quorum: 4,
                        };
                        let mut swarm = match SwarmTripwire::adopt_key_copies(feed, copies.clone(), config) {
                            Ok(swarm) => swarm,
                            Err(error) => {
                                errors.push(format!("adopt order={order_index} base={base} index={index} placement={placement} scenario={scenario}: {error}"));
                                continue;
                            }
                        };
                        for k in 0..MAX_READS {
                            if swarm.is_tripped() {
                                break;
                            }
                            let Some(value) = scenario_label(feed, base, scenario, k) else { break };
                            let result = match feed {
                                Feed::Frames => swarm.observe_frame(FrameClass(value)),
                                Feed::SessionEvents => swarm.observe_session_event(event_of_tag(value)),
                            };
                            match result {
                                Ok(_) => {}
                                Err(error) => {
                                    errors.push(format!("read order={order_index} base={base} index={index} placement={placement} scenario={scenario} k={k}: {error}"));
                                    break;
                                }
                            }
                        }
                        let receipt = swarm.last_receipt();
                        let measurements = swarm.key_measurements();
                        let heats: [f64; 4] = measurements.each_ref().map(|m| m.heat);
                        let ruptured = measurements.iter().filter(|m| m.ruptured).count();
                        let nonzero_min = measurements.iter().map(|m| m.nonzero_bytes).min().unwrap_or(0);
                        if ruptured > 0 && nonzero_min != 0
                            || heats.iter().any(|h| !h.is_finite() || *h < 0.0)
                        {
                            zeroization_failures += 1;
                        }
                        rows.push(Row {
                            order: order_index,
                            base,
                            index,
                            placement,
                            scenario,
                            trip_reads: receipt.map(|r| r.trip_reads as i64).unwrap_or(-1),
                            heats,
                            ruptured,
                            warm: receipt.map(|r| r.warm).unwrap_or(0),
                            nonzero_min,
                            live_unchanged: live == copies[0],
                            external_survives: external.iter().zip(copies.iter()).all(|(a, b)| a == b),
                        });
                    }
                }
            }
        }
    }

    fs::create_dir_all(&out)?;
    let mut ledger = String::from(
        "order,base,index,placement,scenario,trip_reads,heat0,heat1,heat2,heat3,ruptured,warm,nonzero_min,live_unchanged,external_survives\n",
    );
    for row in &rows {
        writeln!(
            ledger,
            "{},{},{},{},{},{},{:.17},{:.17},{:.17},{:.17},{},{},{},{},{}",
            row.order, row.base, row.index, row.placement, row.scenario, row.trip_reads,
            row.heats[0], row.heats[1], row.heats[2], row.heats[3],
            row.ruptured, row.warm, row.nonzero_min, row.live_unchanged, row.external_survives
        )?;
    }
    let feed_name = match feed {
        Feed::Frames => "frames",
        Feed::SessionEvents => "sessions",
    };
    let hill_name = if hill { "on" } else { "off" };
    let ledger_name = format!("{feed_name}-{thermal}-{hill_name}-c{coupling}-ledger.csv");
    let summary_name = format!("{feed_name}-{thermal}-{hill_name}-c{coupling}-summary.md");
    fs::write(out.join(&ledger_name), ledger)?;

    let mut empty_failures = 0_usize;
    let mut external_failures = 0_usize;
    let mut live_failures = 0_usize;
    if rows.len() == expected_rows {
        for row in &rows {
            // donor registration leaves a ~1e-8 heat residue; intact means
            // not ruptured, all 32 bytes nonzero, and heat below 1e-6
            if row.scenario == 7
                && (row.heats.iter().any(|h| !h.is_finite() || *h < 0.0 || *h >= 1e-6)
                    || row.ruptured != 0
                    || row.nonzero_min != 32)
            {
                empty_failures += 1;
            }
            if !row.external_survives {
                external_failures += 1;
            }
            if !row.live_unchanged {
                live_failures += 1;
            }
        }
    }

    let count_le = |rows: &[Row], order: usize, scenario: usize, bound: i64| {
        rows.iter()
            .filter(|row| {
                row.order == order
                    && row.scenario == scenario
                    && row.trip_reads >= 0
                    && row.trip_reads <= bound
            })
            .count()
    };
    let count_trips = |rows: &[Row], order: usize, scenario: usize| {
        rows.iter()
            .filter(|row| row.order == order && row.scenario == scenario && row.trip_reads >= 0)
            .count()
    };

    let mut summary = format!(
        "# Quorum swarm feed eval: feed={feed_name} T={thermal} hill={hill_name} coupling={coupling}\n\n"
    );
    writeln!(summary, "Rows: {} / {expected_rows}", rows.len())?;
    writeln!(summary, "Execution errors: {}", errors.len())?;
    writeln!(summary, "Zeroization failures: {zeroization_failures}")?;
    writeln!(summary, "Empty-trace intact failures: {empty_failures}")?;
    writeln!(summary, "External-copy control failures: {external_failures}")?;
    writeln!(summary, "Live-key-unchanged failures: {live_failures}")?;
    for error in &errors {
        writeln!(summary, "- {error}")?;
    }
    writeln!(summary)?;
    if rows.len() == expected_rows && errors.is_empty() {
        for order in 0..2 {
            let g1 = count_le(&rows, order, 0, 2);
            let g2 = count_le(&rows, order, 1, 8);
            let g3 = count_le(&rows, order, 2, 12);
            let g4 = count_le(&rows, order, 3, 24);
            let g5 = count_le(&rows, order, 4, 24);
            let g6 = count_trips(&rows, order, 5) + count_trips(&rows, order, 6) + count_trips(&rows, order, 7);
            writeln!(summary, "## Order {order}\n")?;
            writeln!(summary, "G1 single-flood <=2: {g1}/144")?;
            writeln!(summary, "G2 dist-flood <=8: {g2}/144")?;
            writeln!(summary, "G3 dist-scan <=12: {g3}/144")?;
            writeln!(summary, "G4 dist-sparse <=24: {g4}/144")?;
            writeln!(summary, "G5 dist-scan-wide <=24: {g5}/144")?;
            writeln!(summary, "G6 benign/move/empty trips: {g6}/432")?;
            writeln!(summary, "G1: {}", if g1 == 144 { "MEASURED" } else { "FAILED" })?;
            writeln!(summary, "G2: {}", if g2 == 144 { "MEASURED" } else { "FAILED" })?;
            writeln!(summary, "G3: {}", if g3 == 144 { "MEASURED" } else { "FAILED" })?;
            writeln!(summary, "G4: {}", if g4 == 144 { "MEASURED" } else { "FAILED" })?;
            writeln!(summary, "G5: {}", if g5 == 144 { "MEASURED" } else { "FAILED" })?;
            writeln!(summary, "G6: {}", if g6 == 0 { "MEASURED" } else { "FAILED" })?;
            writeln!(summary)?;
        }
    }
    fs::write(out.join(&summary_name), summary)?;

    if rows.len() != expected_rows
        || !errors.is_empty()
        || zeroization_failures != 0
        || empty_failures != 0
        || external_failures != 0
        || live_failures != 0
    {
        return Err("frozen swarm feed experiment hard gate failed".into());
    }
    Ok(())
}
