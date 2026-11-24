use std::path::{Path, PathBuf};
use std::process::Command;

use ram_monitor::integrity::{compare_files, compare_records, IntegrityError, RECORD_BYTES, TICKS};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/integrity-bridge/fixtures")
        .join(name)
}

#[test]
fn stored_e013_2202_pair_matches_frozen_native_measurement() {
    let reference =
        include_bytes!("../../../experiments/integrity-bridge/fixtures/2202-reference.bin");
    let clean = include_bytes!("../../../experiments/integrity-bridge/fixtures/2202-clean.bin");
    let modified =
        include_bytes!("../../../experiments/integrity-bridge/fixtures/2202-modified.bin");
    assert_eq!(reference.len(), RECORD_BYTES);
    assert_eq!(reference, clean);
    let unchanged = compare_records(reference, clean).expect("valid stored clean pair");
    assert_eq!(unchanged.ticks.len(), TICKS);
    assert_eq!(unchanged.long_mean, 0.0);
    assert_eq!(unchanged.original_mean, 0.0);
    assert_eq!(unchanged.different_bytes, 0);

    let changed = compare_records(reference, modified).expect("valid stored modified pair");
    assert_eq!(changed.ticks.len(), TICKS);
    assert_eq!(changed.different_bytes, 36);
    assert!((changed.long_mean - 0.10017804602015781).abs() < 1e-11);
    assert!((changed.original_mean - 0.12953510138218877).abs() < 1e-11);
    assert!(changed.peak_mean >= changed.original_mean);
    assert_eq!(
        changed,
        compare_records(reference, modified).expect("exact replay")
    );
}

#[test]
fn all_stored_pairs_and_single_record_are_bounded() {
    for seed in [2201, 2202, 2303, 2304] {
        let reference = fixture(&format!("{seed}-reference.bin"));
        let clean = fixture(&format!("{seed}-clean.bin"));
        let modified = fixture(&format!("{seed}-modified.bin"));
        let clean_receipt = compare_files(&reference, &clean).expect("stored clean fixture");
        assert_eq!(clean_receipt.long_mean, 0.0);
        let changed_receipt = compare_files(&reference, &modified).expect("stored changed fixture");
        assert!(changed_receipt.long_mean.is_finite());
        assert!(changed_receipt.original_mean.is_finite());
        assert!(changed_receipt.peak_mean.is_finite());
        assert!(changed_receipt.different_bytes > 0);
    }
    let single = compare_files(
        &fixture("2202-reference.bin"),
        &fixture("2202-single-record.bin"),
    )
    .expect("single-record fixture");
    assert_eq!(single.different_bytes, 1);
    assert!(single.long_mean.is_finite());
}

#[test]
fn exact_length_and_missing_path_fail_closed() {
    let reference = fixture("2202-reference.bin");
    for (name, size) in [("empty.bin", 0), ("short.bin", 159), ("long.bin", 161)] {
        assert_eq!(
            compare_files(&reference, &fixture(name)).unwrap_err(),
            IntegrityError::Length {
                role: "observed",
                size
            }
        );
    }
    assert!(matches!(
        compare_files(&reference, &fixture("does-not-exist.bin")),
        Err(IntegrityError::Io(_))
    ));
    assert!(matches!(
        compare_files(&reference, &fixture(".")),
        Err(IntegrityError::Io(_))
    ));
}

#[test]
fn cli_runs_integrity_source_and_preserves_rowhammer_source() {
    let exe = env!("CARGO_BIN_EXE_ram-monitor");
    let output = Command::new(exe)
        .args(["--source", "integrity", "--reference-file"])
        .arg(fixture("2202-reference.bin"))
        .arg("--observed-file")
        .arg(fixture("2202-modified.bin"))
        .output()
        .expect("launch CLI");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("UTF-8 CLI output");
    assert_eq!(text.lines().count(), 321);
    assert!(text.lines().next().is_some_and(|line| line
        .contains("integrity_component=E012/E013 observations=320")
        && line.contains("conventional_different_bytes=36")));

    let bad = Command::new(exe)
        .args(["--source", "integrity", "--reference-file"])
        .arg(fixture("2202-reference.bin"))
        .arg("--observed-file")
        .arg(fixture("short.bin"))
        .output()
        .expect("launch CLI with short file");
    assert_eq!(bad.status.code(), Some(3));
    assert!(bad.stdout.is_empty());

    let replay = Command::new(exe)
        .args(["--source", "replay", "--samples", "32"])
        .output()
        .expect("launch existing replay");
    assert!(replay.status.success());
    let replay_text = String::from_utf8_lossy(&replay.stdout);
    assert!(replay_text.contains("samples=") && replay_text.contains("halted=true"));
}
