# No-CPU portable trace result — 2026-11-01

The simulation app no longer contains or exposes CPU/PMU/perf-ring acquisition.
Removed from the active branch:

- `perf.rs`, the perf diagnostic example, and privileged/live perf tests;
- CLI perf source, privilege, all-CPU, physical-key, and sample-period flags;
- GUI perf source and per-CPU/PEBS/ring controls;
- the app crate's direct libc dependency.

The replacement boundary is a portable UTF-8 `time_ns,address` trace. The
std-only parser accepts decimal/hex numbers, comments, blank lines, and
nondecreasing time. It rejects empty, malformed, backward-time, missing-field,
extra-field, and over-one-million-sample input with a line-specific error.

Operational surfaces:

- CLI/AI: `ram-monitor --source trace --trace-file /absolute/path/trace`;
- GUI: starts with no source, accepts drag-and-drop or an entered path, displays
  the selected file/error, and processes only uploaded content;
- generated replay remains available for deterministic demonstrations/tests.

Verification:

- parser valid/invalid unit gates pass;
- CLI uploaded attack processes 130 samples and produces nonzero independent
  rupture receipts; backward-time input exits fail-closed with code 3;
- controller starts with zero samples, then processes only the explicitly loaded
  trace and records its source name/receipt;
- full workspace: 78 passed, zero ignored;
- release GUI stayed alive for the five-second native launch check;
- candidate CLI SHA-256
  `af62ca884d80dbb33ccc27f9ce827b0a0b82e131d053e3dcf57cb9ca025ab740`;
- candidate GUI SHA-256
  `c285f25140791c6314d554bc842621b8a32e4dae53b5a151716123cb4755b583`;
- legacy CPU-ring service is stopped and disabled (`inactive`, `MainPID=0`).

The portable CLI/GUI binaries were installed to `/opt/ram-monitor` with those
exact hashes while the legacy service remained disabled and inactive. Installed
CLI help exposes only `replay` and `trace` sources. Installed hammer replay
produced numerical rupture receipts; scatter replay remained at zero trips. The
installed GUI stayed alive for a second five-second native launch check.

Existing simulation effectiveness gates remain green. This makes the app
portable and removes the unauthorized observation source; it does not convert
simulation evidence into a physical Rowhammer claim.
