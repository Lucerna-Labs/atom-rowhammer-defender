# Atom Rowhammer Defender

**by Lucerna Labs, a division of Lucerna Media**

Private repo: [github.com/Lucerna-Labs/atom-rowhammer-defender](https://github.com/Lucerna-Labs/atom-rowhammer-defender)

Package and binary identifiers remain `ram-monitor` and `ram-monitor-gui`.

This branch is a portable simulation harness. It accepts generated mathematical
observations or an uploaded text trace and feeds concentration, decaying thermal
recurrence, and bounded capacity compositions into independent immutable no-copy
swarms. Runtime never receives a named attack family, expected answer, route,
vote, or classifier.

> **Boundary:** this branch opens no CPU counter, PMU, perf source, or hardware
> ring. A rupture is a simulated emergent numerical response—not a physical
> detection verdict or proof that real memory damage was prevented.

## License

Free for non-commercial use under the [PolyForm Noncommercial License
1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/). Commercial
use requires a separate license from Lucerna Labs.

## Portable trace format

UTF-8 text, one observation per line:

```text
# time_ns,address
0,0x1000
50,0x1040
100,0x1000
```

Decimal and `0x` integers, blank lines, and `#` comments are accepted. The
parser rejects backward time, malformed or missing fields, empty traces, and
traces over one million observations with a line-specific error.

## How it works

```text
uploaded time_ns,address observations
                 │
                 ▼
full mathematical fingerprint identity
                 │
                 ├── exact recurrence memory
                 ├── decaying thermal state and cooling renewal
                 └── bounded three-projection capacity sketch
                 │
                 ▼
independent immutable swarms with owned copies
                 │
                 ▼
numerical rupture receipts
```

The thermal danger boundary remains 128. One fixed class-seven frame is emitted
for each newly reached heat milestone 122 through 128, allowing the immutable
swarm to rupture no later than the simulated boundary without accumulating
unlimited frames on a safe plateau. Full cooling renews only that independent
thermal epoch.

## CLI and AI use

```sh
cargo run -p ram-monitor -- --source trace --trace-file /absolute/path/trace.txt
cargo run -p ram-monitor -- --source replay --scenario hammer --samples 256
```

The trace-file command is the AI-callable boundary. It prints only simulation
counters, provenance, and numerical receipts. It does not connect to another app
or inspect the computer.

## GUI

```sh
cargo run -p ram-monitor-gui --release
```

Drop a trace file anywhere on the window, or enter its absolute path and select
**Load trace**. The GUI starts with no source. **Apply settings** rebuilds the
simulation using the uploaded trace. **Reset** clears and re-arms the current
simulation without capturing or reconnecting anything.

The console uses the Ordo palette and displays ARMED/TRIPPED status, uploaded
source name, simulation counters, evidence provenance, and bounded numerical
receipt log. `RAM_MONITOR_GUI_DEMO=replay` starts a generated demonstration.

## Verification

```sh
cargo test --workspace
```

Current frozen simulation evidence includes:

- 8,704/8,704 known-technique proxies, 0/2,048 controls;
- 2,048/2,048 broader timed attacks, 0/123 controls;
- 5,000/5,000 deterministic fuzz positives, 0/5,000 negatives;
- 55/55 exact-boundary responses;
- recurrence periods through 131,072, 0/30 equal-length unique controls.

See the corresponding precommit, failure, correction, and result files in
`docs/`. These counts establish only the declared simulation scopes.

## Layout

- `crates/ram-monitor/src/trace.rs` — fail-closed portable trace parser/source.
- `crates/ram-monitor/src/replay.rs` — generated simulation source.
- `crates/ram-monitor/src/sketch.rs` — bounded high-cardinality recurrence edge.
- `crates/ram-monitor/src/lib.rs` — exact/thermal composition and immutable feeds.
- `crates/ram-monitor/src/controller.rs` — GUI/headless simulation controller.
- `crates/ram-monitor-gui` — portable upload console.
- `crates/imperium-nocopy-*`, `vendor/nocopy/*` — immutable swarm stack.

The standalone harness arms each swarm with four locally generated owned copies.
No live key is read.
