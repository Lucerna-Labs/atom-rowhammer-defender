use atom_pressure_hop_engine::{Chaos, Engine, Mask, Order, PressureHopOrchestrator};
use ram_monitor::pressure::{read_trace, PressureRecord, RECORDS, TICKS};
use ram_monitor::pressure_hop::evaluate;
use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../experiments");
    if name.starts_with("hop-") {
        root.join("memory-stall-hop-app/fixtures")
            .join(format!("{name}.txt"))
    } else {
        root.join("memory-stall-app/fixtures")
            .join(format!("{name}.txt"))
    }
}

#[test]
fn second_adapter_matches_unchanged_native_at_every_tick() -> Result<(), Box<dyn std::error::Error>>
{
    for name in [
        "hop-l8-733",
        "hop-l16-733",
        "hop-l32-733",
        "quiet-zero",
    ] {
        let records: [PressureRecord; RECORDS] = read_trace(&fixture(name))?;
        let receipt = evaluate(&records)?;
        assert_eq!(receipt, evaluate(&records)?);
        assert_eq!(receipt.ticks.len(), TICKS);
        let mut engine = Engine::new(Order::TransportThenSource, Mask::FULL, Chaos::Plain, 0)?;
        for (tick, pair) in receipt.ticks.iter().zip(records.windows(2)) {
            let dt = pair[1].time_ns - pair[0].time_ns;
            let delta = pair[1].some_total_us - pair[0].some_total_us;
            let fraction = (delta as f64 * 1_000.0) / dt as f64;
            let native = Engine::observe(&mut engine, fraction)?;
            assert_eq!(tick.fraction.to_bits(), fraction.to_bits());
            for (got, expected) in [
                (tick.energy, native.energy),
                (tick.entropy, native.entropy),
                (tick.binding, native.binding),
                (tick.habituation, native.habituation),
                (tick.position, native.position),
                (tick.heat, native.heat),
            ] {
                assert_eq!(
                    got.to_bits(),
                    expected.to_bits(),
                    "{name} tick {}",
                    tick.index
                );
            }
            for (got, expected) in tick.occupation.iter().zip(native.occupation) {
                assert_eq!(got.to_bits(), expected.to_bits(), "{name} occupation");
            }
            for (got, expected) in tick.coherence.iter().zip(native.coherence) {
                assert_eq!(got.to_bits(), expected.to_bits(), "{name} coherence");
            }
            assert_eq!(tick.crossed, native.crossed);
        }
        match name {
            "hop-l16-733" | "hop-l32-733" => assert_eq!(receipt.first_crossing, Some(30)),
            _ => assert_eq!(receipt.first_crossing, None),
        }
    }
    Ok(())
}

#[test]
fn independent_cli_rejects_invalid_files_without_numeric_stdout() {
    let executable = env!("CARGO_BIN_EXE_ram-monitor");
    for name in [
        "empty",
        "short96",
        "long98",
        "malformed",
        "nonmonotone",
        "backwards-counter",
        "too-fast",
        "too-slow",
        "over-interval",
        "oversized",
    ] {
        let output = Command::new(executable)
            .args(["--source", "pressure-hop", "--trace-file"])
            .arg(fixture(name))
            .output()
            .expect("CLI should launch");
        assert_eq!(output.status.code(), Some(3), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    let missing = Command::new(executable)
        .args(["--source", "pressure-hop"])
        .output()
        .expect("CLI should launch");
    assert_eq!(missing.status.code(), Some(2));
    let mixed = Command::new(executable)
        .args(["--source", "pressure-hop", "--trace-file"])
        .arg(fixture("hop-l16-733"))
        .args(["--reference-file", "/dev/null"])
        .output()
        .expect("CLI should launch");
    assert_eq!(mixed.status.code(), Some(2));
}

#[test]
fn first_pressure_source_keeps_its_committed_output() {
    let executable = env!("CARGO_BIN_EXE_ram-monitor");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../experiments/memory-stall-app");
    for name in [
        "dwell501",
        "pulse501",
        "block16-601",
        "quiet-zero",
    ] {
        let output = Command::new(executable)
            .args(["--source", "pressure", "--trace-file"])
            .arg(fixture(name))
            .output()
            .expect("CLI should launch");
        assert!(output.status.success(), "{name}");
        let committed = std::fs::read(root.join(format!("{name}-cli.txt")))
            .expect("committed original pressure output");
        assert_eq!(output.stdout, committed, "{name}");
    }
}
