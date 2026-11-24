//! BUILD-20 gates — the build log is a gated ARTIFACT: complete coverage,
//! required fields, append rule stated. Included at compile time, so the log
//! must exist for this test to build (L3).

use std::collections::BTreeSet;

const LOG: &str = include_str!("../../../docs/BUILD-LOG.md");

/// L1 + L2 + L3 (precommitted: pass) — coverage, structure, artifact.
#[test]
fn gate_log_complete_and_structured() {
    assert!(LOG.contains("Append-only"), "the append rule is stated (L2)");

    let entries: Vec<&str> = LOG.split("## BUILD-").skip(1).collect();
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    for entry in &entries {
        let digits: String = entry.chars().take_while(|c| c.is_ascii_digit()).collect();
        assert!(!digits.is_empty(), "an entry carries no build number");
        let n: u32 = digits.parse().expect("numeric build number");
        assert!(seen.insert(n), "duplicate entry for BUILD-{n}");
        for field in ["**Gates:**", "**Lesson:**", "**Commits:**"] {
            assert!(entry.contains(field), "BUILD-{n} is missing {field}");
        }
    }
    for n in 1..=19u32 {
        assert!(seen.contains(&n), "no entry for BUILD-{n} (L1)");
    }
    eprintln!(
        "build-log: {} entries covering BUILD-{}..BUILD-{}, all with Gates/Lesson/Commits",
        entries.len(),
        seen.first().expect("entries"),
        seen.last().expect("entries")
    );

    assert!(LOG.contains("## E1 "), "the E-series is logged");
    assert!(LOG.contains("## E1R-4 "), "the E-series reaches its sweep");
    assert!(
        LOG.contains("atom-build-protocol"),
        "the habit is carried by the protocol skill, not by memory"
    );
}

/// L4 (added 2026-09-26, same session — Jesse: "and don't forget the log") —
/// ENFORCEMENT: every build doc on disk has a log entry, so a future build
/// that omits its entry fails its own suite instead of relying on memory.
#[test]
fn gate_l4_every_build_doc_has_an_entry() {
    let docs = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let mut documented: BTreeSet<u32> = BTreeSet::new();
    for entry in std::fs::read_dir(&docs).expect("docs directory") {
        let name = entry.expect("dir entry").file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix("BUILD") else {
            continue;
        };
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() || !rest[digits.len()..].starts_with('-') {
            continue; // BUILD-LOG.md and any non-build doc
        }
        documented.insert(digits.parse().expect("numeric build number"));
    }
    assert!(!documented.is_empty(), "no build docs found at {docs:?}");

    let mut covered: BTreeSet<u32> = BTreeSet::new();
    for entry in LOG.split("## BUILD-").skip(1) {
        let digits: String = entry.chars().take_while(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = digits.parse() {
            covered.insert(n);
        }
    }
    let missing: Vec<u32> = documented.difference(&covered).copied().collect();
    assert!(
        missing.is_empty(),
        "build docs without log entries: {missing:?} — append the log (atom-build-protocol step 7)"
    );
    eprintln!(
        "L4: {} build docs on disk, every one logged (through BUILD-{})",
        documented.len(),
        documented.iter().next_back().expect("max")
    );
}
