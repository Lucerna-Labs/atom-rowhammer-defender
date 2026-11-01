#!/usr/bin/env python3
"""Reject the withdrawn CPU/perf collector from the portable trace project."""
from pathlib import Path

root = Path(__file__).resolve().parents[1]
forbidden_files = [
    "crates/ram-monitor/src/perf.rs",
    "crates/ram-monitor/tests/perf_live.rs",
    "crates/ram-monitor/examples/diag.rs",
    "RAM-MONITOR-PRECOMMIT-2026-10-31.md",
    "docs/PMU-HOST-CORRECTION.md",
    "docs/BUGFINDER-REPAIR-PRECOMMIT-2026-10-31.md",
    "docs/LOSS-LOG-CORRECTION-2026-10-31.md",
]
failures = [name for name in forbidden_files if (root / name).exists()]
for folder in (root / "crates/ram-monitor", root / "crates/ram-monitor-gui"):
    for source in folder.rglob("*.rs"):
        text = source.read_text()
        for token in ("perf_event_open", "PerfSource", "PERF_RECORD_SAMPLE", "perf_sources", "mod perf;"):
            if token in text:
                failures.append(f"{source.relative_to(root)}: {token}")
if failures:
    raise SystemExit("Withdrawn CPU implementation found:\n" + "\n".join(failures))
print("Portable trace/replay sources only; no CPU/perf collector.")
