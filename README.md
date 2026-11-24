# Atom Defender

One Linux desktop console for Network Lightcone, Rowhammer trace analysis, File Guard, and Keyboard Guard.
The Overview shows each module independently. Alert history combines trace stops, file alerts and restorations, and unexpected keyboard access.

The existing tools include a portable Rowhammer trace/replay GUI, operator-selected trusted file comparison,
read-only file watch, and an opt-in automatic file restore guard,
read-only file-object continuity, bounded timing sense, two independent
memory-pressure senses, a paired memory/I/O pressure sense, and an
independent fast row-fingerprint phase sense. Run
`python3 scripts/check_portable_only.py` to check this boundary.

**by Lucerna Labs, a division of Lucerna Media**

Repository: [github.com/Lucerna-Labs/atom-rowhammer-defender](https://github.com/Lucerna-Labs/atom-rowhammer-defender)

Package and binary identifiers remain `ram-monitor` and `ram-monitor-gui`.

### Network Lightcone

Open **Network Lightcone**, select a `PATHPACK 1` reference pack, and choose **Start live monitoring** or **Analyze capture**. The initial path points to your local host learner's pack. At start, the worker loads a snapshot and displays its SHA-256. An empty or invalid pack fails visibly. The source file and the installed learner stay under their existing owners.

The worker runs the newer Imperium Door One **exact-path** world and its native Spiderweb strand. Each modeled frame receives a sequenced receipt from the allow world and the reverse-deny world. The panel shows exact matches, reverse-deny matches, other misses and the latest ordered path. The kernel and interface drop counts, incomplete records and missing link metadata are shown separately. This is passive comparison against an unsealed reference pack. A match describes a recorded structural path. Service authorization and host firewall activation are separate decisions.

Live monitoring uses a non-promiscuous Linux cooked capture on `any` in the worker's network namespace. It stops on request or when the dashboard exits. The dedicated admin-group helper receives `CAP_NET_RAW` only to open the capture, then clears its capabilities before loading the pack or processing traffic. The dashboard runs as the desktop user. The application stores summary events in its history. Packet payloads are never written to the history or sent to the dashboard.

Linux cooked SLL and SLL2 captures supply packet direction. Classic Ethernet captures require the operator to select a single known direction for the file. Capture replay uses installed libpcap (PCAP and its supported PCAPNG files). Truncated records have their own count and do not receive an invented path. Kernel observation, checksum offload, capture filters and network namespaces affect what the worker can see.

### Keyboard access alerts

Open **Keyboard Guard → Start monitoring**. The Linux helper observes successful read-capable evdev driver opens for full keyboards with physical sysfs ancestry. It reports the executable name and identity, PID, UID, device and open time. Both read-only and read/write opens qualify. A write-only open, a failed open and an open of a software input device do not. The guard never opens an input device, reads input events or blocks applications. Each arm covers new opens until stopped or the dashboard closes. Existing descriptors, capture inside an application or Xwayland, and hardware loggers are outside this signal.

The unprivileged dashboard recognizes the installed COSMIC compositor and system login service by executable device, inode and UID. Additional exceptions are session-scoped. Unknown opens enter the shared alert history and request a desktop notification (at most one per five seconds). The latest 200 entries are shown; two private, approximately 1 MiB logs are kept under `$XDG_STATE_HOME/atom-defender` (normally `~/.local/state/atom-defender`). Kernel loss and helper failures are visible in the dashboard. **Monitoring** requires a successful kernel attach.

The local amd64 Debian package is built with `packaging/atom-defender/build.sh`. It keeps the existing menu ID and executable paths. Its isolated helper is installed root-owned, executable by the admin group, with only `CAP_BPF` and `CAP_PERFMON` for attachment. After arming it drops its capabilities and sets no-new-privileges. No additional keyboard-device access is granted to the desktop user. The kernel program is the GPL-2.0-only file `crates/atom-keyboard-guard/kernel/keyboard.bpf.c`. Standard builds embed its precompiled little-endian CO-RE object. Rebuild it from the repository root using Clang with the BPF backend:

```sh
clang -target bpfel -O2 -g -fdebug-prefix-map="$PWD"=. -c crates/atom-keyboard-guard/kernel/keyboard.bpf.c -o crates/atom-keyboard-guard/kernel/keyboard.bpf.o
```

The Rowhammer path is a portable simulation harness. It accepts generated mathematical
observations or an uploaded text trace and feeds concentration, decaying thermal
recurrence, and bounded capacity compositions into independent immutable no-copy
swarms. Runtime never receives a named attack family, expected answer, route,
vote, or classifier.

The GUI defaults to **OFF**. Uploading stages a trace without running it. **ON**
starts the staged trace from the beginning; **OFF** stops immediately. Apply and
Reset return the simulation to OFF. The first rupture also halts consumption.

> **Boundary:** the Rowhammer path opens no live hardware counter, hardware performance
> source, or hardware ring. A rupture is a simulated numerical response, not a physical
> detection verdict or proof that real memory damage was prevented.

The separate **experimental** row-phase component accepts an uploaded
4,096-event address trace and returns a coherent numerical phase
response. Its first test covered the old 4,000 ns simulator miss on
generated traces. A broader stress audit found responses on134/256
two-fingerprint and42/256 four-fingerprint rotations, so it is not
a verified one-fingerprint detector. Legitimate hot-cache loops also
respond. It does not replace the original Rowhammer path or establish
physical DRAM rows.

The separate 160-byte file-pair path uses a previously measured E012/E013 Atom
component to report retained numerical evidence of changed content against an
**externally trusted reference**. It can also watch one selected local file
about once per second, alerting on any exact byte change or read failure.
The exact comparison drives the alert because E012/E013 misses sparse changes.
It does not establish reference authenticity or freshness, identify who
changed a file, catch every transient between polls, or prevent writes.
An optional object-identity watch reports same-content replacements of
the selected path separately from the byte count. It requires file birth
time and can alert on legitimate replacements as well.

The separate **automatic restore guard** acts on one explicitly armed local
file after a completed selected-name write event or fallback check. It captures trusted bytes
once, preserves the changed bytes and file metadata in a private evidence
directory, restores the selected path, and verifies exact trusted bytes before
reporting success. The pinned Atom cognitive engine chooses a repair method
from verified successes; unknown or conflicting memory uses a fixed atomic
fallback. Exact content and object changes always trigger action regardless of
that choice. This is post-event file repair, not pre-write blocking, writer
identification, or physical Rowhammer protection. Its learning lasts for the
armed session. See `docs/COGNITIVE-AUTO-RESTORE-RESULT-2026-11-14.md`.

The armed guard now runs as one **Spiderweb Bus Next strand**. Its typed
outcome is delivered through a subscriber to the CLI or GUI after the entire
preserve/restore/verify transaction finishes. A bus message never decides
whether an exact file-integrity alert should be acted on. The pinned bus
source is copied into this checkout as regular files with its license notices;
the guard has no sibling-checkout or symlink dependency. See
`docs/SPIDERWEB-GUARD-RESULT-2026-11-14.md`.

The file-object path records and later compares device, inode and birth time.
It can reveal a same-content atomic replacement that a content-only comparison
cannot distinguish. This is a conventional supporting sensor, not an Atom
stack or proof that a replacement was unauthorized.

The timing path accepts one uploaded 64-event connection stream and reports
fundamental/overtone material values. Its fresh test separated moderate jitter
from exact/alternating timers on the declared generated traces. Some shuffled
streams and ordinary small-jitter timers also responded, so it does not
identify command-and-control intent or protect a live network.

The pressure path accepts one uploaded 96-interval memory PSI history and
reports coherent/material state. Its synthetic test responded to sustained
high stall fractions, while short equal-count pulses stayed below capacity.
Ordinary rolling and run-length checks matched or exceeded this bounded task.
It does not identify an attacker, sample the live system, or relieve pressure.
The separate two-hop stack responds to shorter repeated high-stall episodes
on its own frozen synthetic corpus. Both return their own numerical state;
neither selects or combines an attack verdict.
The paired memory/I/O stack responds to simultaneous high stalls while
staying quiet on the declared matched anti-phase and partial-overlap traces.
It also reports numerical state, without inferring a cause.

## License

Atom Defender and the bundled E012/E013, Atom Cognitive Engine, Spiderweb,
and Imperium components are available under the [MIT License](LICENSE),
including for commercial use. Preserve the copyright and license notices
when redistributing them. The keyboard BPF kernel program is GPL-2.0-only.
External dependencies retain their respective licenses.

## Portable trace format

UTF-8 text, one observation per line:

```text
# time_ns,address
0,0x1000
50,0x1040
100,0x1000
```

Decimal and `0x` integers, blank lines, and `#` comments are accepted. The
parser rejects backward time, malformed or missing fields, empty traces,
files over 64 MiB, nonregular files, and traces over one million
observations with a line-specific sample-count error.

For timestamped uploads, the CLI prints two model results after the legacy replay
summary and any stop receipt. The GUI displays both when the operator turns the
trace on. Both readouts cover the full upload (up to `--samples` on the CLI),
including observations after the legacy replay stops.

| Readout | What the reported sample means |
| --- | --- |
| Timing, 50 µs | A repeat is a visit to the same full fingerprint within 50,000 ns, inclusive. The CLI reports the first repeat and first rupture of a separately armed Atom swarm. The GUI shows the repeat count and first swarm rupture. |
| Heat, 128 units | A repeat within 4 ms deposits one unit. Heat decays with a 500 µs time constant. The readout gives peak heat, the first sample at or above 128, and the first rupture of the heat lane's own Atom swarm. |

Both histories retain up to 65,536 prior observations. Samples are numbered
from one. These readouts describe the uploaded mathematical addresses. The
existing replay controls automatic stopping. Choosing a new stop model is pending.

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

## Fast row phase trace

Provide exactly 4,096 uploaded `time_ns,address` observations using the
portable trace syntax above, with strictly increasing timestamps. The
independent two-crate phase material uses a fixed `address>>6` mathematical
fingerprint and reports every numeric tick, its first capacity crossing,
peak/final material and phase power:

```sh
cargo run -p ram-monitor -- --source row-phase --trace-file /absolute/path/trace.txt
```

This experimental readout is separate from the existing Rowhammer
ON/OFF simulation and swarms. Its initial bounded result and the later
one-fingerprint specificity failure are in
`docs/ROWHAMMER-PHASE-MEMORY-RESULT-2026-11-14.md` and
`docs/ROW-PHASE-HASH-STRESS-RESULT-2026-11-14.md`.

## Trusted file comparison

Choose two **exactly 160-byte** files. The app reads them once and does not
change either file:

```sh
cargo run -p ram-monitor -- --source integrity \
  --reference-file /absolute/path/trusted-reference.bin \
  --observed-file /absolute/path/observed.bin
```

The first line reports the E012/E013 material mean in its long and original
windows, the peak mean, and an exact conventional byte-difference count.
The following 320 lines report all four material outputs and their mean at
each observation. The count is a separate ordinary comparison; it is never
an Atom input or Atom verdict. The bridge rejects missing files and sizes
other than 160 bytes. See `docs/INTEGRITY-BRIDGE-PRECOMMIT-2026-11-14.md`
and its measured result for the exact tested boundary.

To keep checking the current observed path against reference bytes captured
once at startup:

```sh
cargo run -p ram-monitor -- --source integrity-watch \
  --reference-file /absolute/path/trusted-reference.bin \
  --observed-file /absolute/path/observed.bin
```

This runs until interrupted. It prints and flushes a scalar line immediately
and about every second. `--watch-interval-ms N` accepts 100..60000 ms;
`--watch-checks N` makes a finite run. A finite run exits 0 if clean and 4
if any exact difference or later read failure occurred. Bad files at startup
exit 3 before numeric output. This is detection only; the reference is not
reloaded, and no file is changed by the watch.

Add `--watch-object` to capture the observed file's device, inode and birth
time at startup and report `object_changed` on each check. Both bytes and
object identity come from the same opened file. A same-content atomic
replacement alerts in this mode even when `conventional_different_bytes=0`.
The content-only default remains available on file systems without birth
time. Object changes are not evidence of unauthorized activity.

To automatically restore **one selected 160-byte file**, create a private
evidence directory on the same filesystem as the observed file, then arm the
guard explicitly:

```sh
mkdir -m 700 /absolute/path/private-evidence
cargo run -p ram-monitor -- --source integrity-guard \
  --reference-file /absolute/path/trusted-reference.bin \
  --observed-file /absolute/path/observed.bin \
  --evidence-dir /absolute/path/private-evidence
```

No per-event confirmation is needed. `--watch-interval-ms N` sets the
100..60000 ms fallback recheck; selected-name events can wake the guard
earlier. An open selected write remains pending until close. `--watch-checks N`
makes a finite test run. The guard refuses initial
invalid paths, follows no selected-file symlink, and stops with an error if it
cannot preserve evidence or verify restoration. Evidence may contain sensitive
file contents; the selected directory must be owned by the current user and
mode 0700. Only generated 160-byte files have been exercised for this mode.

## File object continuity

Capture a regular file's current object token, retain it, and later check the
current pathname or a renamed/hardlinked path against that token:

```sh
cargo run -p ram-monitor -- --source object-capture --observed-file /absolute/path/file
cargo run -p ram-monitor -- --source object-check --observed-file /absolute/path/file \
  --identity-token DEVICE:INODE:BIRTH_NS
```

Both commands only read metadata and print a point-in-time observation.
Capture prints `object_identity=DEVICE:INODE:BIRTH_NS`; check prints
`object_continuity=same` or `different`. A token is user-held data and is
not authenticated. Rename and hardlink keep the same object; a copy or
atomic replacement creates another. In-place edits keep object identity,
so use the separate trusted content comparison for content questions.
See `docs/FILE-OBJECT-CONTINUITY-BRIDGE-RESULT-2026-11-14.md`.

## Timing trace sense

Provide exactly 64 strictly increasing observations for one externally
scoped connection pair, in UTF-8 `time_ns,bytes_out` format:

```text
# time_ns,bytes_out
0,64
100000000,64
...
```

Run the fixed four-mode Atom composition without a live network collector:

```sh
cargo run -p ram-monitor -- --source rhythm --trace-file /absolute/path/rhythm.txt
```

The CLI prints its numerical first crossing, peak/final energy, fixed time
scale, and all 64 fundamental/overtone tick values. The app never turns a
crossing into an attacker verdict or firewall action. The measured function
is bounded to the generated 64-event tests in
`docs/JITTER-EXACT-HEARTBEAT-CONFIRM-RESULT-2026-11-14.md`.

## Memory-pressure trace sense

Provide exactly 97 lines of `time_ns,some_total_us` with no header. Both
columns are unsigned decimal integers from one monotonic clock and one
Linux memory PSI `some total` counter; consecutive samples must be 0.9–1.1
seconds apart. The app derives 96 stall fractions and rejects inconsistent
counters or excessive stall time before reporting any numerical output.

```text
0,0
1000000000,0
2000000000,800000
...
```

```sh
cargo run -p ram-monitor -- --source pressure --trace-file /absolute/path/pressure.txt
```

The output contains all 96 material ticks, first capacity crossing, peak
fraction, and peak/final energy. The only confirmed function is a bounded
synthetic 32-tick high-stall dwell response; a legitimate sustained workload
can look identical. See `docs/MEMORY-STALL-MATERIAL-RESULT-2026-11-14.md`.

Run the second, independent qutrit material on the same strict input format:

```sh
cargo run -p ram-monitor -- --source pressure-hop --trace-file /absolute/path/pressure.txt
```

The qutrit component's fresh test responded to 32/32 repeated 16-tick high
stall profiles and 0/32 matched short 8-tick controls, including 13 cases
missed by the first pressure component. A plain rolling/run-length rule
matched that separation. Its failed first version and the corrected
result are preserved in `docs/MEMORY-STALL-HOP-V1-RESULT-2026-11-14.md`
and `docs/MEMORY-STALL-HOP-V2-RESULT-2026-11-14.md`.

## Paired memory and I/O pressure

Provide exactly 97 `time_ns,memory_some_total_us,io_some_total_us` decimal
records with no header. The two cumulative Linux PSI `some total` counters
must be sampled against one monotonic clock, 0.9–1.1 seconds apart. Each
counter must be monotone and its delta must fit the elapsed interval.

```sh
cargo run -p ram-monitor -- --source psi-coupling --trace-file /absolute/path/paired-psi.txt
```

The CLI prints one summary and all 96 joint material ticks. Its fresh
synthetic test found 32/32 complete simultaneous high histories and 0/32
anti-phase, half-overlap and single-resource controls. A plain
simultaneous-run rule matched the main separation; legitimate work can
also cause joint pressure. The app reads only the uploaded file and
does not sample `/proc/pressure` itself. See
`docs/COUPLED-PSI-MATERIAL-RESULT-2026-11-14.md`.

## GUI

```sh
cargo run -p ram-monitor-gui --release
```

Drop a trace file anywhere on the window, or enter its absolute path and select
**Load trace**. The GUI starts OFF with no source. Loading only stages the file.
Use the large **TURN ON / TURN OFF** button to start from the beginning or stop
immediately. **Apply settings** and **Reset** return the simulation to OFF without
capturing or reconnecting anything.

Expand **Trusted file comparison (160 bytes)** in the main panel to select a
reference and observed file and press **Compare files**. It displays the
numerical E012/E013 material values separately from the exact byte count.
**Start watch** captures the reference and checks the observed path about once
per second; **Stop watch** ends polling and preserves the last receipt.
Select **Also watch file object identity** before Start to alert on
same-content replacements of the selected path.
To enable automatic response, enter an existing private evidence directory and
press **ARM AUTOMATIC RESTORE**. The guard runs in a background worker and
reports each verified repair. **Disarm automatic restore** stops it. This is
independent of the Rowhammer simulation ON/OFF control.
Expand **File object continuity (read only)** to capture the current object
in GUI memory and check its identity later, without turning on Rowhammer.
Expand **Experimental row phase trace (4096 events)** to evaluate its
independent numerical material without turning on Rowhammer.
Expand **Timing trace (64 events)** to evaluate a separate uploaded timing
stream. Its result does not change the Rowhammer ON/OFF state.
Expand **Memory pressure trace (96 intervals)** for the separate PSI-counter
file and numerical material result; it also leaves Rowhammer OFF.
Expand **Memory pressure short episodes (96 intervals)** to run the second
numeric material independently on the same file format.
Expand **Memory + I/O pressure (96 intervals)** for the separate paired
counter file and four-state numeric material.

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
`docs/`. These counts establish only the declared simulation scopes. They were
recorded by the frozen evaluators and are not re-derived from the shipped crate;
the shipped detector's timed band is stated in the 2026-11-02 correction at the
top of `docs/ADVERSARIAL-TIMED-RESULT.md`.

## Layout

- `crates/ram-monitor/src/trace.rs` — fail-closed portable trace parser/source.
- `crates/ram-monitor/src/row_phase.rs` — strict 4,096-event upload bridge
  into the independent complex phase material.
- `crates/atom-row-phase-*` — driven phase/material primitives and fixed
  engine; physical-row and intent limits are retained in `docs/`.
- `crates/ram-monitor/src/replay.rs` — generated simulation source.
- `crates/ram-monitor/src/sketch.rs` — bounded high-cardinality recurrence edge.
- `crates/ram-monitor/src/lib.rs` — exact/thermal composition and immutable feeds.
- `crates/ram-monitor/src/controller.rs` — GUI/headless simulation controller.
- `crates/ram-monitor/src/integrity.rs` — exact-size byte-pair bridge into the
  unchanged E012/E013 engine.
- `crates/ram-monitor/src/integrity_watch.rs` — captured-reference, read-only
  repeated comparison with exact-content and optional object-change alerts.
- `crates/ram-monitor/src/auto_restore.rs` — opt-in evidence-first file
  actuator and exact postcondition verification.
- `crates/ram-monitor-cognitive` — bounded repair-method selection through
  the pinned Atom cognitive engine in `vendor/cognitive-engine-upstream`.
- `crates/ram-monitor-file-events` — event-time selected-path wake with a
  bounded periodic fallback.
- `crates/ram-monitor-bus` — Spiderweb-owned guard strand and typed outcome
  subscriber shared by the CLI and GUI.
- `vendor/spiderweb-bus-upstream` — pinned Spiderweb core/supervisor source,
  copied as regular files with license notices.
- `crates/ram-monitor/src/file_object.rs` — conventional device/inode/birth-time
  observation and strict user-held identity token.
- `crates/ram-monitor/src/rhythm.rs` — strict portable timing parser and fixed
  bridge into the four-mode numerical stack.
- `crates/atom-rhythm-four-*` — measured narrow timing mechanics and fixed
  engine; failed broad-beacon and stress results remain in `docs/`.
- `crates/ram-monitor/src/pressure.rs` — strict uploaded PSI-counter trace
  validation and numerical pressure receipt.
- `crates/atom-pressure-*` — intrinsic two-level density/material pressure
  history component.
- `crates/ram-monitor/src/pressure_hop.rs` — second independent numeric
  bridge over the strict pressure trace parser.
- `crates/atom-pressure-hop-*` — corrected three-site, two-hop material
  component; the failed first revision remains in Git and experiment data.
- `crates/ram-monitor/src/psi_coupling.rs` — strict paired memory/I/O PSI
  counter upload and numerical joint-state receipt.
- `crates/atom-psi-coupling-*` — four-state coupled pressure material;
  classical comparators and bounded synthetic evidence are retained.
- `crates/ram-monitor-gui` — portable upload console.
- `crates/imperium-nocopy-*`, `vendor/nocopy/*` — immutable swarm stack.
- `vendor/integrity-e013` — preserved two-crate statistical integrity
  component; see its provenance and source-license note.

The standalone harness arms each swarm with four locally generated owned copies.
No live key is read.

### Credential file access (Linux)

In **File Guard → Credential file access**, enter the absolute path of a user-owned
regular file and the PID of the running application allowed to open it. Arm the
guard and authenticate the installed helper through PolicyKit. Wait for **ARMED**.
New opens of that inode, including hard links, require the pinned process and
executable identity. Other processes receive a kernel denial on their first
open. Each attempt feeds the vendored Atom no-copy sensor. Its reaction and
tripwire appear alongside cumulative allow and deny counts. The tripwire holds
a one-byte sentinel. The helper never reads the credential contents.

Coverage lasts while the permission group is alive. Disarming, closing Defender,
losing the helper, or exiting the approved process releases the mark. Replacing
the file creates a new inode outside the mark and requires rearming. Already-open
descriptors, copies, sidecar files, and activity inside the approved process are
outside this boundary. Another process with authority to terminate the guard can
end protection. Kernel queue loss or an unsupported event ends the session.
