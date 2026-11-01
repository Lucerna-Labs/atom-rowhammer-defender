//! Live-feed swarm eval harness. Drives `SwarmLiveFeed::observe_wire_frame` over
//! the frozen precommit scenario class sequences (wire_len = smallest length of
//! that class per the quantize table) for
//! 16 indices x 3 placements x orders L/M, writing a CSV ledger + summary per run.
//!
//! Usage:
//!   cargo run -p imperium-nocopy-swarm-live --example swarm_live_eval -- \
//!       --out <dir> [--hill on|off] [--coupling 0.30]
//!
//! Test-only synthetic key bytes; no real memory or live key is touched.

use imperium_nocopy_swarm_live::{Order, SwarmLiveConfig, SwarmLiveFeed, SWARM_CELLS};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

// wire_len for class c: smallest length in that class per the quantize table.
fn len_of(class: u8) -> usize {
    [1, 65, 129, 257, 513, 1025, 1201, 1473][class as usize]
}

// scenario codes: 0=flood, 1=dist-flood, 2=dist-scan, 3=benign, 4=move, 5=mixed-long
fn classes_for(scenario: usize, k: usize) -> Option<u8> {
    match scenario {
        0 => (k < 16).then_some(0),
        1 => (k < 16).then_some((k % 4) as u8),
        2 => (k < 16).then_some((4 + k % 4) as u8),
        3 => (k < 4).then_some(k as u8),
        4 => (k < 4).then_some(7),
        5 => (k < 32).then_some((k % 8) as u8),
        _ => None,
    }
}

fn splitmix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

// Test-only synthetic bytes, same construction as the swarm eval.
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

struct Row {
    order: usize,
    index: usize,
    placement: usize,
    scenario: usize,
    trip_reads: i64,
    heats: [f64; SWARM_CELLS],
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut out: Option<PathBuf> = None;
    let mut hill = true;
    let mut coupling = 0.30_f64;
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--out" => out = Some(PathBuf::from(iter.next().ok_or("--out dir")?)),
            "--hill" => {
                hill = match iter.next().ok_or("--hill value")?.as_str() {
                    "on" => true,
                    "off" => false,
                    other => return Err(format!("unknown hill {other}").into()),
                }
            }
            "--coupling" => coupling = iter.next().ok_or("--coupling value")?.parse::<f64>()?,
            _ => return Err(format!("unknown argument {arg}").into()),
        }
    }
    let out = out.ok_or("--out required")?;
    fs::create_dir_all(&out)?;
    let orders = [Order::L, Order::M];
    let expected_rows = 2 * 16 * 3 * 6;

    let mut rows = Vec::with_capacity(expected_rows);
    let mut errors = Vec::new();
    for (order_index, order) in orders.into_iter().enumerate() {
        for index in 0..16usize {
            for placement in 0..3usize {
                let copies: [Vec<u8>; SWARM_CELLS] =
                    core::array::from_fn(|j| synthetic_key(0, (index + j) % 16));
                let seeds: [u64; SWARM_CELLS] = core::array::from_fn(|j| {
                    splitmix((((index + j) % 16) * 31 + placement) as u64)
                });
                for scenario in 0..6 {
                    let mut config = SwarmLiveConfig::s3([0, 1, 2, 3], seeds, order);
                    config.hill = hill;
                    config.coupling = coupling;
                    let mut feed = SwarmLiveFeed::new();
                    if let Err(error) = feed.adopt_key_copies(copies.clone(), config) {
                        errors.push(format!(
                            "adopt order={order_index} index={index} placement={placement} scenario={scenario}: {error}"
                        ));
                        continue;
                    }
                    let mut trip_reads = -1_i64;
                    let mut heats = [0.0_f64; SWARM_CELLS];
                    for k in 0..32usize {
                        let Some(class) = classes_for(scenario, k) else {
                            break;
                        };
                        let wire_len = len_of(class);
                        if let Some(receipt) = feed.observe_wire_frame(wire_len) {
                            trip_reads = receipt.trip_reads as i64;
                            heats = receipt.heats;
                            break;
                        }
                    }
                    if trip_reads < 0 {
                        if let Some(current) = feed.heats() {
                            heats = current;
                        }
                    }
                    rows.push(Row {
                        order: order_index,
                        index,
                        placement,
                        scenario,
                        trip_reads,
                        heats,
                    });
                }
            }
        }
    }

    let tag = format!("live-{}-c{coupling}", if hill { "on" } else { "off" });
    let ledger_path = out.join(format!("{tag}-ledger.csv"));
    let mut csv = String::from("order,index,placement,scenario,trip_reads,heat0,heat1,heat2,heat3\n");
    for row in &rows {
        let _ = writeln!(
            csv,
            "{},{},{},{},{},{:.17},{:.17},{:.17},{:.17}",
            row.order,
            row.index,
            row.placement,
            row.scenario,
            row.trip_reads,
            row.heats[0],
            row.heats[1],
            row.heats[2],
            row.heats[3]
        );
    }
    fs::write(&ledger_path, &csv)?;

    let mut summary = String::new();
    let _ = writeln!(
        summary,
        "# Live-feed swarm eval: hill={} coupling={coupling}",
        if hill { "on" } else { "off" }
    );
    let _ = writeln!(summary, "\nRows: {}/{} ", rows.len(), expected_rows);
    let _ = writeln!(summary, "Execution errors: {}", errors.len());
    for error in &errors {
        let _ = writeln!(summary, "- {error}");
    }
    // Frozen gates per order: L2 s0<=2, L3 s1<=8, L4 s2<=12, L5 s3 trips 0,
    // L6 s4 trips 0, L7 s5<=8 (48 configs each).
    let bounds = [(0usize, 2i64), (1, 8), (2, 12), (5, 8)];
    for order_index in 0..2 {
        let _ = writeln!(summary, "\n## Order {order_index}\n");
        for (scenario, bound) in bounds {
            let v: Vec<i64> = rows
                .iter()
                .filter(|r| r.order == order_index && r.scenario == scenario)
                .map(|r| r.trip_reads)
                .collect();
            let n = v.len();
            let le = v.iter().filter(|t| **t >= 0 && **t <= bound).count();
            let mx = v.iter().copied().filter(|t| *t >= 0).max().unwrap_or(-1);
            let _ = writeln!(
                summary,
                "L{} s{scenario} <= {bound}: {le}/{n} (max {mx})",
                scenario + 2
            );
        }
        for scenario in [3usize, 4] {
            let v: Vec<i64> = rows
                .iter()
                .filter(|r| r.order == order_index && r.scenario == scenario)
                .map(|r| r.trip_reads)
                .collect();
            let trips = v.iter().filter(|t| **t >= 0).count();
            let _ = writeln!(summary, "L{} s{scenario} trips: {trips}/{} (want 0)", scenario + 2, v.len());
        }
    }
    let summary_path = out.join(format!("{tag}-summary.md"));
    fs::write(&summary_path, &summary)?;
    println!("wrote {ledger_path:?} and {summary_path:?} ({} rows, {} errors)", rows.len(), errors.len());
    Ok(())
}
