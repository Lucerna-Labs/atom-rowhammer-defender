use atom_psi_coupling_engine::{Chaos, CouplingOrchestrator, Engine, Event, Mask, Order};
use ram_monitor::psi_coupling::{evaluate, read_trace, PsiRecord, RECORDS, TICKS};
use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/psi-coupling-app/fixtures")
        .join(format!("{name}.txt"))
}

#[test]
fn paired_adapter_matches_native_at_every_tick() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "positive",
        "anti-phase",
        "half-overlap",
        "memory-only",
        "io-only",
        "quiet",
    ] {
        let records: [PsiRecord; RECORDS] = read_trace(&fixture(name))?;
        let receipt = evaluate(&records)?;
        assert_eq!(receipt, evaluate(&records)?);
        assert_eq!(receipt.ticks.len(), TICKS);
        let mut engine = Engine::new(Order::TransportThenSource, Mask::FULL, Chaos::Plain, 0)?;
        for (tick, pair) in receipt.ticks.iter().zip(records.windows(2)) {
            let dt = pair[1].time_ns - pair[0].time_ns;
            let memory = (pair[1].memory_some_total_us - pair[0].memory_some_total_us) as f64
                * 1000.0
                / dt as f64;
            let io =
                (pair[1].io_some_total_us - pair[0].io_some_total_us) as f64 * 1000.0 / dt as f64;
            let state = Engine::observe(
                &mut engine,
                Event {
                    first: memory,
                    second: io,
                },
            )?;
            assert_eq!(tick.memory_fraction.to_bits(), memory.to_bits());
            assert_eq!(tick.io_fraction.to_bits(), io.to_bits());
            for (got, expected) in [
                (tick.energy, state.energy),
                (tick.joint_coherence, state.coherence_03),
                (tick.entropy, state.entropy),
                (tick.binding, state.binding),
                (tick.habituation, state.habituation),
                (tick.position, state.position),
                (tick.heat, state.heat),
            ] {
                assert_eq!(
                    got.to_bits(),
                    expected.to_bits(),
                    "{name} tick {}",
                    tick.index
                );
            }
            for (got, expected) in tick.occupation.iter().zip(state.occupation) {
                assert_eq!(got.to_bits(), expected.to_bits(), "{name} occupation");
            }
            assert_eq!(tick.crossed, state.crossed);
        }
        if name == "positive" {
            assert_eq!(receipt.first_crossing, Some(63));
        } else {
            assert_eq!(receipt.first_crossing, None, "{name}");
        }
    }
    Ok(())
}

#[test]
fn strict_cli_has_no_partial_output_for_bad_paired_traces() {
    let executable = env!("CARGO_BIN_EXE_ram-monitor");
    for name in [
        "empty",
        "short96",
        "long98",
        "malformed",
        "nonmonotone",
        "backwards-memory",
        "backwards-io",
        "too-fast",
        "too-slow",
        "over-memory",
        "over-io",
        "oversized",
    ] {
        let output = Command::new(executable)
            .args(["--source", "psi-coupling", "--trace-file"])
            .arg(fixture(name))
            .output()
            .expect("CLI should launch");
        assert_eq!(output.status.code(), Some(3), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    let directory = Command::new(executable)
        .args(["--source", "psi-coupling", "--trace-file"])
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .output()
        .expect("CLI should launch");
    assert_eq!(directory.status.code(), Some(3));
    assert!(directory.stdout.is_empty());
}

#[test]
fn paired_source_requires_explicit_inputs() {
    let executable = env!("CARGO_BIN_EXE_ram-monitor");
    let missing = Command::new(executable)
        .args(["--source", "psi-coupling"])
        .output()
        .expect("CLI should launch");
    assert_eq!(missing.status.code(), Some(2));
    let mixed = Command::new(executable)
        .args(["--source", "psi-coupling", "--trace-file"])
        .arg(fixture("positive"))
        .args(["--reference-file", "/dev/null"])
        .output()
        .expect("CLI should launch");
    assert_eq!(mixed.status.code(), Some(2));
    let valid = Command::new(executable)
        .args(["--source", "psi-coupling", "--trace-file"])
        .arg(fixture("positive"))
        .output()
        .expect("CLI should launch");
    assert!(valid.status.success());
    assert_eq!(valid.stdout.split(|byte| *byte == b'\n').count(), TICKS + 2);
    let text = String::from_utf8_lossy(&valid.stdout);
    assert!(text.starts_with("psi_coupling_component=four-state-material intervals=96"));
    assert!(text.contains("first_crossing=63"));
}
