use std::path::PathBuf;
use std::io::Write;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use imperium_nocopy_swarm_live::{Order, SwarmLiveConfig, SwarmLiveFeed};
use ram_monitor::{file_object::{identity_token, parse_identity_token, read_object}, integrity::compare_files, pressure::{evaluate as evaluate_pressure, read_trace as read_pressure_trace}, pressure_hop::evaluate as evaluate_pressure_hop, psi_coupling::{evaluate as evaluate_psi_coupling, read_trace as read_psi_trace}, rhythm::{evaluate as evaluate_rhythm, read_trace as read_rhythm_trace}, row_phase::{evaluate as evaluate_row_phase, read_upload as read_row_phase_upload}, read_trace, AccessSource, RamMonitor, ReplaySource, RowGeometry, TraceSource};
use ram_monitor::integrity_watch::{IntegrityWatch, WatchCheck};
use ram_monitor::auto_restore::{AutomaticFileGuard, GuardOutcome};
use ram_monitor_bus::run_to_stdout;
use ram_monitor::row_models::{TraceModels, HEAT_CAPACITY, HORIZON_NS};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SourceKind {
    Replay,
    Trace,
    Integrity,
    IntegrityWatch,
    IntegrityGuard,
    Rhythm,
    Pressure,
    PressureHop,
    ObjectCapture,
    ObjectCheck,
    PsiCoupling,
    RowPhase,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum Scenario {
    Hammer,
    Scatter,
    Mixed,
    Benign,
}

#[derive(Parser, Debug)]
#[command(version, about = "Portable Rowhammer, file, timing and resource-pressure observations")]
struct Args {
    #[arg(long, value_enum, default_value = "replay")]
    source: SourceKind,
    #[arg(long)]
    trace_file: Option<PathBuf>,
    #[arg(long)]
    reference_file: Option<PathBuf>,
    #[arg(long)]
    observed_file: Option<PathBuf>,
    #[arg(long)]
    evidence_dir: Option<PathBuf>,
    #[arg(long)]
    identity_token: Option<String>,
    #[arg(long)]
    watch_checks: Option<u64>,
    #[arg(long)]
    watch_interval_ms: Option<u64>,
    #[arg(long)]
    watch_object: bool,
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
        if !matches!(self.source, SourceKind::IntegrityWatch | SourceKind::IntegrityGuard)
            && (self.watch_checks.is_some() || self.watch_interval_ms.is_some() || self.watch_object) {
            return Err("watch options require --source integrity-watch or integrity-guard");
        }
        if !matches!(self.source, SourceKind::IntegrityGuard) && self.evidence_dir.is_some() {
            return Err("--evidence-dir requires --source integrity-guard");
        }
        if matches!(self.source, SourceKind::ObjectCapture | SourceKind::ObjectCheck) {
            if self.observed_file.is_none() {
                return Err("object observation requires --observed-file");
            }
            if self.reference_file.is_some() || self.trace_file.is_some() {
                return Err("object observation accepts only --observed-file and an optional check token");
            }
            return match self.source {
                SourceKind::ObjectCapture if self.identity_token.is_some() =>
                    Err("--identity-token requires --source object-check"),
                SourceKind::ObjectCheck if self.identity_token.is_none() =>
                    Err("--source object-check requires --identity-token"),
                _ => Ok(()),
            };
        }
        if self.identity_token.is_some() {
            return Err("--identity-token requires --source object-check");
        }
        if matches!(self.source, SourceKind::IntegrityWatch) {
            if self.reference_file.is_none() || self.observed_file.is_none() {
                return Err("--source integrity-watch requires --reference-file and --observed-file");
            }
            if self.trace_file.is_some() {
                return Err("--source integrity-watch does not accept --trace-file");
            }
            if self.watch_checks.is_some_and(|count| count == 0 || count > 100_000) {
                return Err("--watch-checks must be in 1..=100000");
            }
            if self.watch_interval_ms.is_some_and(|ms| !(100..=60_000).contains(&ms)) {
                return Err("--watch-interval-ms must be in 100..=60000");
            }
            return Ok(());
        }
        if matches!(self.source, SourceKind::IntegrityGuard) {
            if self.reference_file.is_none() || self.observed_file.is_none() || self.evidence_dir.is_none()
                || self.trace_file.is_some() || self.watch_object {
                return Err("integrity-guard needs reference, observed, evidence directory, and no trace/token");
            }
            if self.watch_checks.is_some_and(|n| n == 0 || n > 100_000)
                || self.watch_interval_ms.is_some_and(|ms| !(100..=60_000).contains(&ms)) {
                return Err("guard checks or fallback interval outside allowed bounds");
            }
            return Ok(());
        }
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
        match self.source {
            SourceKind::Trace if self.trace_file.is_none() =>
                Err("--source trace requires --trace-file"),
            SourceKind::Rhythm if self.trace_file.is_none() =>
                Err("--source rhythm requires --trace-file"),
            SourceKind::Pressure if self.trace_file.is_none() =>
                Err("--source pressure requires --trace-file"),
            SourceKind::PressureHop if self.trace_file.is_none() =>
                Err("--source pressure-hop requires --trace-file"),
            SourceKind::PsiCoupling if self.trace_file.is_none() =>
                Err("--source psi-coupling requires --trace-file"),
            SourceKind::RowPhase if self.trace_file.is_none() =>
                Err("--source row-phase requires --trace-file"),
            SourceKind::RowPhase if self.row_shift != 6 || self.row_bits != 8 =>
                Err("--source row-phase uses fixed 64-byte fingerprint geometry"),
            SourceKind::Integrity if self.reference_file.is_none() || self.observed_file.is_none() =>
                Err("--source integrity requires --reference-file and --observed-file"),
            SourceKind::Integrity if self.trace_file.is_some() =>
                Err("--trace-file requires a trace, rhythm or pressure source"),
            SourceKind::Trace | SourceKind::Rhythm | SourceKind::Pressure | SourceKind::PressureHop | SourceKind::PsiCoupling | SourceKind::RowPhase | SourceKind::Replay
                if self.reference_file.is_some() || self.observed_file.is_some() =>
                Err("reference and observed files require --source integrity"),
            SourceKind::Replay if self.trace_file.is_some() =>
                Err("--trace-file requires a trace, rhythm or pressure source"),
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
    if matches!(args.source, SourceKind::Integrity) {
        run_integrity(&args);
        return;
    }
    if matches!(args.source, SourceKind::IntegrityWatch) {
        run_integrity_watch(&args);
        return;
    }
    if matches!(args.source, SourceKind::IntegrityGuard) {
        run_integrity_guard(&args);
        return;
    }
    if matches!(args.source, SourceKind::Rhythm) {
        run_rhythm(&args);
        return;
    }
    if matches!(args.source, SourceKind::Pressure) {
        run_pressure(&args);
        return;
    }
    if matches!(args.source, SourceKind::PressureHop) {
        run_pressure_hop(&args);
        return;
    }
    if matches!(args.source, SourceKind::ObjectCapture) {
        run_object_capture(&args);
        return;
    }
    if matches!(args.source, SourceKind::ObjectCheck) {
        run_object_check(&args);
        return;
    }
    if matches!(args.source, SourceKind::PsiCoupling) {
        run_psi_coupling(&args);
        return;
    }
    if matches!(args.source, SourceKind::RowPhase) {
        run_row_phase(&args);
        return;
    }
    let geometry = RowGeometry {
        row_shift: args.row_shift,
        row_bits: args.row_bits,
    };
    let mut monitor = RamMonitor::new(geometry, args.window, args.thresh);
    let mut feed = armed_feed();
    let mut model_report = None;
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
            let limit = samples.len().min(args.samples as usize);
            model_report = Some(match TraceModels::new(geometry).and_then(|mut models| {
                models.extend(&samples[..limit])?;
                Ok(models.report())
            }) {
                Ok(report) => report,
                Err(error) => {
                    eprintln!("ram-monitor: {error}");
                    std::process::exit(3);
                }
            });
            match TraceSource::new(samples) {
                Ok(source) => Box::new(source),
                Err(error) => {
                    eprintln!("ram-monitor: {error}");
                    std::process::exit(3);
                }
            }
        }
        SourceKind::Integrity | SourceKind::IntegrityWatch | SourceKind::IntegrityGuard => {
            eprintln!("ram-monitor: integrity source reached the address monitor");
            std::process::exit(2);
        }
        SourceKind::Rhythm => {
            eprintln!("ram-monitor: rhythm source reached the address monitor");
            std::process::exit(2);
        }
        SourceKind::Pressure => {
            eprintln!("ram-monitor: pressure source reached the address monitor");
            std::process::exit(2);
        }
        SourceKind::PressureHop => {
            eprintln!("ram-monitor: pressure-hop source reached the address monitor");
            std::process::exit(2);
        }
        SourceKind::ObjectCapture | SourceKind::ObjectCheck => {
            eprintln!("ram-monitor: object source reached the address monitor");
            std::process::exit(2);
        }
        SourceKind::PsiCoupling => {
            eprintln!("ram-monitor: psi-coupling source reached the address monitor");
            std::process::exit(2);
        }
        SourceKind::RowPhase => {
            eprintln!("ram-monitor: row-phase source reached the address monitor");
            std::process::exit(2);
        }
    };
    let stats = monitor.run_until_rupture(&mut feed, source.as_mut(), args.samples);
    let trips = feed.stats().trips + monitor.secondary_trips();
    println!(
        "samples={} frames={} trips={} halted={} armed={} evidence={}",
        stats.samples,
        feed.frames() + monitor.secondary_frames(),
        trips,
        stats.halted,
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
    if let Some(report) = model_report {
        let index = |n: Option<u64>| n.map_or_else(|| "none".to_string(), |v| v.to_string());
        let r = report.reading;
        println!("timing_model samples={} horizon_ns={} repeats={} first_repeat={} first_swarm_rupture={}",
            r.samples, HORIZON_NS, r.timing_matches, index(r.first_match), index(report.timing_rupture));
        println!("heat_model samples={} capacity={} peak_heat={:.12} first_crossing={} first_swarm_rupture={}",
            r.samples, HEAT_CAPACITY, r.peak_heat, index(r.first_heat_crossing), index(report.heat_rupture));
    }
}

fn run_integrity(args: &Args) {
    let (Some(reference), Some(observed)) =
        (args.reference_file.as_deref(), args.observed_file.as_deref()) else {
        eprintln!("ram-monitor: missing integrity files");
        std::process::exit(2);
    };
    let receipt = match compare_files(reference, observed) {
        Ok(receipt) => receipt,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    println!(
        "integrity_component=E012/E013 observations={} long_mean={:.17e} original_mean={:.17e} peak_mean={:.17e} conventional_different_bytes={}",
        receipt.ticks.len(), receipt.long_mean, receipt.original_mean,
        receipt.peak_mean, receipt.different_bytes
    );
    for tick in receipt.ticks {
        println!(
            "tick={} materials={:.17e},{:.17e},{:.17e},{:.17e} mean={:.17e}",
            tick.index, tick.materials[0], tick.materials[1],
            tick.materials[2], tick.materials[3], tick.mean
        );
    }
}

fn emit_watch_check(checks: u64, check: &WatchCheck) {
    match check {
        WatchCheck::Compared(receipt) => println!(
            "watch_check={} alert={} conventional_different_bytes={} atom_long_mean={:.17e}",
            checks, check.alert(), receipt.different_bytes, receipt.long_mean
        ),
        WatchCheck::ComparedWithObject(receipt, changed) => println!(
            "watch_check={} alert={} conventional_different_bytes={} object_changed={} atom_long_mean={:.17e}",
            checks, check.alert(), receipt.different_bytes, changed, receipt.long_mean
        ),
        WatchCheck::ReadError(error) => println!(
            "watch_check={} alert=true read_error={:?}", checks, error
        ),
        WatchCheck::EngineError(error) => println!(
            "watch_check={} alert=true engine_error={:?}", checks, error
        ),
    }
    let _ = std::io::stdout().flush();
}

fn run_integrity_watch(args: &Args) {
    let (Some(reference), Some(observed)) =
        (args.reference_file.as_deref(), args.observed_file.as_deref()) else {
        eprintln!("ram-monitor: missing integrity watch files");
        std::process::exit(2);
    };
    let started = if args.watch_object {
        IntegrityWatch::start_with_object(reference, observed)
    } else {
        IntegrityWatch::start(reference, observed)
    };
    let mut watch = match started {
        Ok(watch) => watch,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    let interval = Duration::from_millis(args.watch_interval_ms.unwrap_or(1000));
    loop {
        emit_watch_check(watch.checks(), watch.current());
        if args.watch_checks.is_some_and(|limit| watch.checks() >= limit) {
            break;
        }
        std::thread::sleep(interval);
        watch.poll();
    }
    if watch.ever_alerted() {
        std::process::exit(4);
    }
}

fn run_integrity_guard(args: &Args) {
    let (Some(reference), Some(observed), Some(evidence)) =
        (args.reference_file.as_deref(), args.observed_file.as_deref(), args.evidence_dir.as_deref())
    else { std::process::exit(2) };
    let mut guard = match AutomaticFileGuard::arm(reference, observed, evidence) {
        Ok(guard) => guard,
        Err(error) => { eprintln!("ram-monitor: {error}"); std::process::exit(3) }
    };
    println!("guard_armed observed={} evidence={}", observed.display(), evidence.display());
    let _ = std::io::stdout().flush();
    let interval = Duration::from_millis(args.watch_interval_ms.unwrap_or(1000));
    let result = run_to_stdout(
        move |duration| guard.wait_and_act(duration).map_err(|error| error.to_string()),
        interval,
        args.watch_checks, |outcome| matches!(outcome, GuardOutcome::PendingWrite),
        |checks, outcome| match outcome {
            GuardOutcome::Clean { signal } => format!("guard_check={checks} clean signal={signal:?}"),
            GuardOutcome::PendingWrite => format!("guard_check={checks} pending_write awaiting_close"),
            GuardOutcome::Restored(receipt) => format!(
                "guard_check={checks} restored method={:?} selection={:?} first_failed={} changed_bytes={} object_changed={} evidence={} learned={} elapsed_us={}",
                receipt.executed, receipt.selected.source, receipt.first_attempt_failed,
                receipt.changed_bytes, receipt.object_changed, receipt.evidence.display(),
                receipt.cognitive_recorded, receipt.elapsed.as_micros()
            ),
        },
    );
    if let Err(error) = result {
        eprintln!("ram-monitor: {error}");
        std::process::exit(4);
    }
}

fn run_rhythm(args: &Args) {
    let Some(path) = args.trace_file.as_deref() else {
        eprintln!("ram-monitor: missing timing trace");
        std::process::exit(2);
    };
    let receipt = read_rhythm_trace(path).and_then(|events| evaluate_rhythm(&events));
    let receipt = match receipt {
        Ok(receipt) => receipt,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    println!(
        "rhythm_component=four-mode events={} tau_ns={:.17e} first_crossing={} peak_energy={:.17e} final_energy={:.17e}",
        receipt.ticks.len(), receipt.tau_ns,
        receipt.first_crossing.unwrap_or(0), receipt.peak_energy,
        receipt.final_energy
    );
    for tick in receipt.ticks {
        println!(
            "tick={} energy={:.17e} purity={:.17e} fundamental={:.17e} overtone={:.17e} binding={:.17e} adaptation={:.17e} strains={:.17e},{:.17e} heats={:.17e},{:.17e}",
            tick.index, tick.energy, tick.purity, tick.fundamental,
            tick.overtone, tick.binding, tick.adaptation,
            tick.strain[0], tick.strain[1], tick.heat[0], tick.heat[1]
        );
    }
}

fn run_pressure(args: &Args) {
    let Some(path) = args.trace_file.as_deref() else {
        eprintln!("ram-monitor: missing pressure trace");
        std::process::exit(2);
    };
    let receipt = read_pressure_trace(path).and_then(|records| evaluate_pressure(&records));
    let receipt = match receipt {
        Ok(receipt) => receipt,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    let crossing = receipt.first_crossing.map_or("none".to_string(), |n| n.to_string());
    println!(
        "pressure_component=two-level-material intervals={} first_crossing={} peak_fraction={:.17e} peak_energy={:.17e} final_energy={:.17e}",
        receipt.ticks.len(), crossing, receipt.peak_fraction,
        receipt.peak_energy, receipt.final_energy
    );
    for tick in receipt.ticks {
        println!(
            "tick={} fraction={:.17e} energy={:.17e} crossed={} population={:.17e} coherence={:.17e} entropy={:.17e} binding={:.17e} habituation={:.17e} position={:.17e} heat={:.17e}",
            tick.index, tick.fraction, tick.energy, tick.crossed,
            tick.population, tick.coherence, tick.entropy,
            tick.binding, tick.habituation, tick.position, tick.heat
        );
    }
}

fn run_pressure_hop(args: &Args) {
    let Some(path) = args.trace_file.as_deref() else {
        eprintln!("ram-monitor: missing pressure-hop trace");
        std::process::exit(2);
    };
    let receipt = read_pressure_trace(path).and_then(|records| evaluate_pressure_hop(&records));
    let receipt = match receipt {
        Ok(receipt) => receipt,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    let crossing = receipt.first_crossing.map_or("none".to_string(), |n| n.to_string());
    println!(
        "pressure_hop_component=three-site-material intervals={} first_crossing={} peak_fraction={:.17e} peak_energy={:.17e} final_energy={:.17e}",
        receipt.ticks.len(), crossing, receipt.peak_fraction,
        receipt.peak_energy, receipt.final_energy
    );
    for tick in receipt.ticks {
        println!(
            "tick={} fraction={:.17e} energy={:.17e} crossed={} occupations={:.17e},{:.17e},{:.17e} coherences={:.17e},{:.17e},{:.17e} entropy={:.17e} binding={:.17e} habituation={:.17e} position={:.17e} heat={:.17e}",
            tick.index, tick.fraction, tick.energy, tick.crossed,
            tick.occupation[0], tick.occupation[1], tick.occupation[2],
            tick.coherence[0], tick.coherence[1], tick.coherence[2],
            tick.entropy, tick.binding, tick.habituation, tick.position, tick.heat
        );
    }
}

fn run_object_capture(args: &Args) {
    let Some(path) = args.observed_file.as_deref() else {
        eprintln!("ram-monitor: missing observed file");
        std::process::exit(2);
    };
    let reading = match read_object(path) {
        Ok(reading) => reading,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    println!(
        "object_identity={} links={} size={}",
        identity_token(reading.identity), reading.links, reading.size
    );
}

fn run_object_check(args: &Args) {
    let (Some(path), Some(raw)) =
        (args.observed_file.as_deref(), args.identity_token.as_deref()) else {
        eprintln!("ram-monitor: missing observed file or identity token");
        std::process::exit(2);
    };
    let baseline = match parse_identity_token(raw) {
        Ok(identity) => identity,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    let reading = match read_object(path) {
        Ok(reading) => reading,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    let relation = if baseline == reading.identity { "same" } else { "different" };
    println!(
        "object_continuity={} baseline={} observed={} links={} size={}",
        relation, identity_token(baseline), identity_token(reading.identity),
        reading.links, reading.size
    );
}

fn run_psi_coupling(args: &Args) {
    let Some(path) = args.trace_file.as_deref() else {
        eprintln!("ram-monitor: missing paired PSI trace");
        std::process::exit(2);
    };
    let receipt = read_psi_trace(path).and_then(|records| evaluate_psi_coupling(&records));
    let receipt = match receipt {
        Ok(receipt) => receipt,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    let crossing = receipt.first_crossing.map_or("none".to_string(), |n| n.to_string());
    println!(
        "psi_coupling_component=four-state-material intervals={} first_crossing={} peak_memory_fraction={:.17e} peak_io_fraction={:.17e} peak_energy={:.17e} final_energy={:.17e}",
        receipt.ticks.len(), crossing, receipt.peak_memory_fraction,
        receipt.peak_io_fraction, receipt.peak_energy, receipt.final_energy
    );
    for tick in receipt.ticks {
        println!(
            "tick={} memory_fraction={:.17e} io_fraction={:.17e} energy={:.17e} crossed={} occupations={:.17e},{:.17e},{:.17e},{:.17e} joint_coherence={:.17e} entropy={:.17e} binding={:.17e} habituation={:.17e} position={:.17e} heat={:.17e}",
            tick.index, tick.memory_fraction, tick.io_fraction, tick.energy, tick.crossed,
            tick.occupation[0], tick.occupation[1], tick.occupation[2], tick.occupation[3],
            tick.joint_coherence, tick.entropy, tick.binding,
            tick.habituation, tick.position, tick.heat
        );
    }
}

fn run_row_phase(args: &Args) {
    let Some(path) = args.trace_file.as_deref() else {
        eprintln!("ram-monitor: missing row-phase trace");
        std::process::exit(2);
    };
    let receipt = read_row_phase_upload(path).and_then(|samples| evaluate_row_phase(&samples));
    let receipt = match receipt {
        Ok(receipt) => receipt,
        Err(error) => {
            eprintln!("ram-monitor: {error}");
            std::process::exit(3);
        }
    };
    let crossing = receipt.first_crossing.map_or("none".to_string(), |n| n.to_string());
    println!(
        "row_phase_component=complex-phase-memory events={} first_crossing={} peak_energy={:.17e} final_energy={:.17e} peak_power={:.17e} final_power={:.17e}",
        receipt.ticks.len(), crossing, receipt.peak_energy, receipt.final_energy,
        receipt.peak_power, receipt.final_power
    );
    for tick in receipt.ticks {
        println!(
            "tick={} time_ns={} fingerprint={} energy={:.17e} crossed={} power={:.17e} real={:.17e} imag={:.17e} entropy={:.17e} binding={:.17e} habituation={:.17e} position={:.17e} heat={:.17e} alpha={:.17e}",
            tick.index, tick.time_ns, tick.fingerprint, tick.energy,
            tick.crossed, tick.power, tick.real, tick.imag, tick.entropy,
            tick.binding, tick.habituation, tick.position, tick.heat, tick.alpha
        );
    }
}
