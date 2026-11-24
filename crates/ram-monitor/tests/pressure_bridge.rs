use atom_pressure_engine::{Chaos, Engine, Mask, Order, PressureOrchestrator};
use ram_monitor::pressure::{evaluate, read_trace, PressureRecord, RECORDS, TICKS};
use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/memory-stall-app/fixtures")
        .join(format!("{name}.txt"))
}

#[test]
fn adapter_matches_native_state_for_every_tick() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "dwell501",
        "pulse501",
        "block16-601",
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
                (tick.population, native.population),
                (tick.coherence, native.coherence),
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
            assert_eq!(tick.crossed, native.crossed);
        }
        match name {
            "dwell501" => assert_eq!(receipt.first_crossing, Some(51)),
            "pulse501" | "quiet-zero" => {
                assert_eq!(receipt.first_crossing, None)
            }
            "block16-601" => assert_eq!(receipt.first_crossing, Some(64)),
            _ => unreachable!(),
        }
    }
    Ok(())
}

#[test]
fn strict_cli_rejects_all_invalid_fixtures_without_partial_stdout() {
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
            .args(["--source", "pressure", "--trace-file"])
            .arg(fixture(name))
            .output()
            .expect("CLI should launch");
        assert_eq!(output.status.code(), Some(3), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    let directory = Command::new(executable)
        .args(["--source", "pressure", "--trace-file"])
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .output()
        .expect("CLI should launch");
    assert_eq!(directory.status.code(), Some(3));
    assert!(directory.stdout.is_empty());
}

#[test]
fn pressure_source_is_explicit_and_independent() {
    let executable = env!("CARGO_BIN_EXE_ram-monitor");
    let missing = Command::new(executable)
        .args(["--source", "pressure"])
        .output()
        .expect("CLI should launch");
    assert_eq!(missing.status.code(), Some(2));
    let invalid_mix = Command::new(executable)
        .args(["--source", "pressure", "--trace-file"])
        .arg(fixture("dwell501"))
        .args(["--reference-file", "/dev/null"])
        .output()
        .expect("CLI should launch");
    assert_eq!(invalid_mix.status.code(), Some(2));
    let valid = Command::new(executable)
        .args(["--source", "pressure", "--trace-file"])
        .arg(fixture("dwell501"))
        .output()
        .expect("CLI should launch");
    assert!(valid.status.success());
    assert_eq!(valid.stdout.split(|byte| *byte == b'\n').count(), TICKS + 2);
    let text = String::from_utf8_lossy(&valid.stdout);
    assert!(text.starts_with("pressure_component=two-level-material intervals=96"));
    assert!(text.contains("first_crossing=51"));
}
