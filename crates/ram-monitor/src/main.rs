use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use imperium_nocopy_swarm_live::{Order, SwarmLiveConfig, SwarmLiveFeed};
use ram_monitor::{read_trace, AccessSource, RamMonitor, ReplaySource, RowGeometry, TraceSource};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SourceKind {
    Replay,
    Trace,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Scenario {
    Hammer,
    Scatter,
    Mixed,
    Benign,
}

#[derive(Parser, Debug)]
#[command(version, about = "Portable simulation-only Rowhammer trace harness")]
struct Args {
    #[arg(long, value_enum, default_value = "replay")]
    source: SourceKind,
    #[arg(long)]
    trace_file: Option<PathBuf>,
    #[arg(long, value_enum, default_value = "hammer")]
    scenario: Scenario,
    #[arg(long, default_value = "0")]
    hammer_class: u8,
    #[arg(long, default_value = "1000000")]
    samples: u64,
    #[arg(long, default_value = "64")]
    window: usize,
    #[arg(long, default_value = "16")]
    thresh: usize,
    #[arg(long, default_value = "6")]
    row_shift: u32,
    #[arg(long, default_value = "8")]
    row_bits: u32,
}

impl Args {
    fn validate(&self) -> Result<(), &'static str> {
        if self.window == 0 || self.window > 1_000_000 {
            return Err("window must be in 1..=1000000");
        }
        if self.thresh == 0 || self.thresh > self.window {
            return Err("thresh must be in 1..=window");
        }
        if self.row_shift >= 64 || self.row_bits == 0 || self.row_bits > 64 - self.row_shift {
            return Err("row geometry must use a shift below 64 and valid remaining bits");
        }
        if self.hammer_class > 7 {
            return Err("hammer-class must be in 0..=7");
        }
        if self.samples == 0 || self.samples > 1_000_000 {
            return Err("samples must be in 1..=1000000");
        }
        match (self.source, self.trace_file.is_some()) {
            (SourceKind::Trace, false) => Err("--source trace requires --trace-file"),
            (SourceKind::Replay, true) => Err("--trace-file requires --source trace"),
            _ => Ok(()),
        }
    }
}

fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn armed_feed() -> SwarmLiveFeed {
    let mut feed = SwarmLiveFeed::new();
    let seeds = core::array::from_fn(|index| mix(index as u64 ^ 0x5eed));
    feed.set_arming_config(SwarmLiveConfig::s3([0, 1, 2, 3], seeds, Order::L));
    for label in 0..4_u64 {
        let copy = (0..32)
            .map(|position| (mix(label * 32 + position ^ 0xc0ffee) & 255) as u8)
            .collect();
        let _ = feed.ingest_key_copy(label, copy);
    }
    feed
}

fn main() {
    let args = Args::parse();
    if let Err(error) = args.validate() {
        eprintln!("ram-monitor: invalid arguments: {error}");
        std::process::exit(2);
    }
    let geometry = RowGeometry {
        row_shift: args.row_shift,
        row_bits: args.row_bits,
    };
    let mut monitor = RamMonitor::new(geometry, args.window, args.thresh);
    let mut feed = armed_feed();
    let mut source: Box<dyn AccessSource> = match args.source {
        SourceKind::Replay => {
            let replay = match args.scenario {
                Scenario::Hammer => {
                    ReplaySource::hammer(args.hammer_class, args.samples as usize, args.row_shift)
                }
                Scenario::Scatter => ReplaySource::scatter(args.samples as usize, args.row_shift),
                Scenario::Mixed => ReplaySource::mixed(args.samples as usize, args.row_shift),
                Scenario::Benign => ReplaySource::benign(args.samples as usize, args.row_shift),
            };
            Box::new(replay)
        }
        SourceKind::Trace => {
            let Some(path) = args.trace_file.as_deref() else {
                eprintln!("ram-monitor: missing trace file");
                std::process::exit(2);
            };
            let samples = match read_trace(path) {
                Ok(samples) => samples,
                Err(error) => {
                    eprintln!("ram-monitor: {error}");
                    std::process::exit(3);
                }
            };
            match TraceSource::new(samples) {
                Ok(source) => Box::new(source),
                Err(error) => {
                    eprintln!("ram-monitor: {error}");
                    std::process::exit(3);
                }
            }
        }
    };
    let stats = monitor.run(&mut feed, source.as_mut(), args.samples);
    let trips = feed.stats().trips + monitor.secondary_trips();
    println!(
        "samples={} frames={} trips={} armed={} evidence={}",
        stats.samples,
        feed.frames() + monitor.secondary_frames(),
        trips,
        feed.is_armed() && monitor.secondary_armed(),
        monitor.evidence.summary()
    );
    if let Some(receipt) = feed
        .receipts()
        .last()
        .copied()
        .or_else(|| monitor.last_secondary_receipt())
    {
        println!(
            "receipt trip_reads={} warm={} ruptured={} heats={:?}",
            receipt.trip_reads, receipt.warm, receipt.ruptured, receipt.heats
        );
    }
}
