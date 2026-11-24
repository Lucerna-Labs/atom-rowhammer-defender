use std::path::{Path, PathBuf};
use std::process::Command;

use ram_monitor::rhythm::{evaluate, parse_trace, read_trace, RhythmError, EVENTS};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/rhythm-app/fixtures")
        .join(name)
}

#[test]
fn stored_fresh_rhythm_traces_keep_their_narrow_numeric_scope() {
    let jitter = read_trace(&fixture("jitter20.txt")).expect("valid jitter trace");
    let first = evaluate(&jitter).expect("native rhythm run");
    assert_eq!(first.ticks.len(), EVENTS);
    assert_eq!(first.first_crossing, Some(27));
    assert!((first.peak_energy - 0.1861807820248022).abs() < 1e-11);
    assert_eq!(first, evaluate(&jitter).expect("exact repeat"));

    for name in ["exact.txt", "alternating.txt", "shuffled20.txt"] {
        let rows = read_trace(&fixture(name)).expect("valid control trace");
        assert_eq!(
            evaluate(&rows).expect("native control run").first_crossing,
            None
        );
    }
    let scheduled = read_trace(&fixture("small-jitter5.txt")).expect("valid witness");
    assert_eq!(
        evaluate(&scheduled).expect("native witness").first_crossing,
        Some(36)
    );
}

#[test]
fn parser_is_strict_and_accepts_hex_numbers() {
    let original = std::fs::read_to_string(fixture("jitter20.txt")).expect("fixture text");
    let hex = original.replacen("0,64", "0x0,0x40", 1);
    assert_eq!(parse_trace(&hex).expect("hex trace")[0].bytes_out, 64);
    for name in [
        "empty.txt",
        "short63.txt",
        "long65.txt",
        "backward.txt",
        "malformed.txt",
    ] {
        assert!(read_trace(&fixture(name)).is_err(), "{name}");
    }
    assert_eq!(
        parse_trace("0,64\n0,64\n"),
        Err(RhythmError::Line {
            line: 2,
            reason: "timestamp must increase"
        })
    );
}

#[test]
fn cli_uses_the_native_engine_and_fails_without_partial_output() {
    let exe = env!("CARGO_BIN_EXE_ram-monitor");
    let output = Command::new(exe)
        .args(["--source", "rhythm", "--trace-file"])
        .arg(fixture("jitter20.txt"))
        .output()
        .expect("launch app CLI");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("UTF-8 numeric output");
    assert_eq!(text.lines().count(), 65);
    assert!(text
        .lines()
        .next()
        .is_some_and(|line| line.contains("rhythm_component=four-mode events=64")
            && line.contains("first_crossing=27")));
    let bad = Command::new(exe)
        .args(["--source", "rhythm", "--trace-file"])
        .arg(fixture("short63.txt"))
        .output()
        .expect("launch malformed app CLI");
    assert_eq!(bad.status.code(), Some(3));
    assert!(bad.stdout.is_empty());
}
