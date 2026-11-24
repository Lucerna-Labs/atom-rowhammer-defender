use atom_row_phase_engine::{Chaos, Engine, Event, Mask, Order, RowPhaseOrchestrator};
use ram_monitor::row_phase::{evaluate, read_upload, EVENTS};
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/row-phase-app/fixtures")
        .join(format!("{name}.txt"))
}

#[test]
fn uploaded_row_phase_matches_native_every_tick() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "target-1301",
        "slow-1301",
        "unique-1301",
        "benign-identical-1301",
        "round8-false-1326",
    ] {
        let samples = read_upload(&fixture(name))?;
        let receipt = evaluate(&samples)?;
        assert_eq!(receipt, evaluate(&samples)?);
        assert_eq!(receipt.ticks.len(), EVENTS);
        let mut engine = Engine::new(Order::PhaseInformationFirst, Mask::FULL, Chaos::Plain, 0)?;
        for (sample, tick) in samples.iter().zip(&receipt.ticks) {
            let time_ns = sample.time_ns.expect("uploaded times are required");
            let fingerprint = sample.addr >> 6;
            let native = Engine::observe(
                &mut engine,
                Event {
                    time_ns,
                    fingerprint,
                },
            )?;
            assert_eq!(tick.time_ns, time_ns);
            assert_eq!(tick.fingerprint, fingerprint);
            for (got, expected) in [
                (tick.energy, native.energy),
                (tick.power, native.power),
                (tick.real, native.real),
                (tick.imag, native.imag),
                (tick.entropy, native.entropy),
                (tick.binding, native.binding),
                (tick.habituation, native.habituation),
                (tick.position, native.position),
                (tick.heat, native.heat),
                (tick.alpha, native.alpha),
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
            "target-1301" | "benign-identical-1301" => {
                assert_eq!(receipt.first_crossing, Some(75));
            }
            "round8-false-1326" => assert_eq!(receipt.first_crossing, Some(170)),
            _ => assert_eq!(receipt.first_crossing, None),
        }
    }
    Ok(())
}

#[test]
fn row_phase_cli_is_explicit_and_rejects_bad_uploads_without_stdout() {
    let exe = env!("CARGO_BIN_EXE_ram-monitor");
    for name in [
        "empty",
        "short4095",
        "long4097",
        "malformed",
        "equal-time",
        "backward-time",
    ] {
        let output = Command::new(exe)
            .args(["--source", "row-phase", "--trace-file"])
            .arg(fixture(name))
            .output()
            .expect("CLI launches");
        assert_eq!(output.status.code(), Some(3), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    let missing = Command::new(exe)
        .args(["--source", "row-phase"])
        .output()
        .expect("CLI launches");
    assert_eq!(missing.status.code(), Some(2));
    let wrong_geometry = Command::new(exe)
        .args(["--source", "row-phase", "--trace-file"])
        .arg(fixture("target-1301"))
        .args(["--row-shift", "7"])
        .output()
        .expect("CLI launches");
    assert_eq!(wrong_geometry.status.code(), Some(2));
    let valid = Command::new(exe)
        .args(["--source", "row-phase", "--trace-file"])
        .arg(fixture("target-1301"))
        .output()
        .expect("CLI launches");
    assert!(valid.status.success());
    assert_eq!(
        valid.stdout.split(|byte| *byte == b'\n').count(),
        EVENTS + 2
    );
    let text = String::from_utf8_lossy(&valid.stdout);
    assert!(text.starts_with("row_phase_component=complex-phase-memory events=4096"));
    assert!(text.contains("first_crossing=75"));
}

#[test]
fn trace_preserves_legacy_output_and_appends_both_model_results() {
    let exe = env!("CARGO_BIN_EXE_ram-monitor");
    let output = Command::new(exe)
        .args(["--source", "trace", "--trace-file"])
        .arg(fixture("target-1301"))
        .args(["--samples", "4096"])
        .output()
        .expect("CLI launches");
    assert!(output.status.success());
    let committed = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/row-phase-app/old-rowhammer-target.txt"),
    )
    .expect("committed old Rowhammer output");
    let extra = output.stdout.strip_prefix(committed.as_slice()).expect("unchanged legacy bytes");
    let extra = std::str::from_utf8(extra).expect("numeric text");
    let rows: Vec<_> = extra.lines().collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], "timing_model samples=4096 horizon_ns=50000 repeats=4095 first_repeat=2 first_swarm_rupture=8");
    assert!(rows[1].starts_with("heat_model samples=4096 capacity=128 peak_heat="));
    assert!(rows[1].ends_with(" first_crossing=none first_swarm_rupture=none"));
}
