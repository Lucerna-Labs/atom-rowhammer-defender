# Spiderweb Bus Next is wired into the automatic guard

The user requested an operational Spiderweb integration with no symlinks.
`ram-monitor-bus` now owns the guard's in-process `Bus` and `Strand`. The
existing `AutomaticFileGuard::wait_and_act` runs completely inside that
strand: selected event, exact recheck, changed-byte preservation, cognitive
method choice, restoration, trusted-byte postcondition, and durable result
receipt all finish before a typed bus outcome is published. The CLI and GUI
subscribe to the exact outcome socket **before** the live strand is registered.
The bridge registers `GuardDelivery<GuardOutcome>` in Spiderweb's clone
registry and reconciles published and received counts on join. A missing
subscriber therefore fails the bridge's delivery gate even though raw
Spiderweb `publish` would return success. The bus is the runtime and user
status path, but no bus message authorizes or suppresses a file repair.

On guard failure, the strand publishes a terminal error and detaches; a
panic becomes a lifecycle crash, is reported to the caller and ends the
spiderless bus run. Automatic restart is deliberately absent because it
could recapture an altered observed file as trusted. GUI Disarm cooperatively
stops and joins the bus worker. The copied kernel's nine production stdout
logs were redirected to stderr so the CLI retains `guard_armed` first and
machine-readable `guard_check` lines. The upstream source is unchanged.

## Source identity and no-symlink boundary

`scripts/import_spiderweb_source.py` pinned the clean Spiderweb Bus Next
checkout at `f083e85555da750fcf325da98d41eabff274bc47` and copied 19
tracked files, 347,713 bytes, as regular files under
`vendor/spiderweb-bus-upstream/`. This includes complete `spiderweb` and
`spider` crates, their tests, LICENSE, COMMERCIAL.md, and
THIRD_PARTY_NOTICES.md. A generated two-member workspace manifest replaces
the upstream manifest's unrelated members. The only production source
transformation is the nine stdout-to-stderr logging calls, with original and
patched hashes recorded in
`experiments/spiderweb-source-import/SOURCE-VERIFICATION.json`.
The import is idempotent, verifies its pinned source and all hashes, and
finds zero symlinks in the copied tree. The defender uses a relative Cargo
path to that real source copy, not a symlink or external checkout at runtime.

## Measured boundary

- Copied `spiderweb` and `spider` offline suite: 103 passed, 0 failed.
- Defender root `cargo test --workspace --locked --offline --quiet`:
  157 passed, 0 failed, 2 existing ignored. The bridge tests cover three
  typed deliveries, no-subscriber detection and forced-strand-crash
  reporting. One GUI guard test arms a generated file, restores one edit
  through its bus subscription, joins after Disarm, and confirms a later
  edit remains changed. A second waits until a terminal bus failure is
  queued before polling, proving the GUI drains final deliveries after join
  and preserves the failure status during Disarm. The portable-only guard
  and focused clippy pass.
- The existing generated CLI probe passed all six automatic repairs through
  the bus strand, including a slow 161-byte write held pending until close,
  a same-content object replacement, and a missing file. Changed bytes and
  `.result` receipts remained; evidence-preservation failure left its file
  changed; an unrelated sibling caused no action. Raw bus-run output is
  `experiments/spiderweb-guard/CLI-RESULT.json`.
- A finite clean CLI run printed exactly one `guard_armed` and two
  `guard_check` stdout lines; kernel diagnostics went to stderr. Receipt:
  `experiments/spiderweb-guard/STDOUT-RESULT.json`. The first test harness
  falsely rejected its own path spelling, retained in
  `STDOUT-FIRST-ATTEMPT.json`. The bridge's first test used an unsuffixed
  integer and registered the wrong concrete type; its failure is retained
  in `BRIDGE-FIRST-ATTEMPT.json` and the corrected test passes.
- A finite `--watch-checks 1` run during an open selected write printed
  `pending_write`, exited 4 with an unresolved-write error, and made no
  evidence or restoration action. Receipt:
  `experiments/spiderweb-guard/PENDING-FINITE-RESULT.json`.
- Five alternating generated one-byte edits per mode, using the same guard
  code, gave direct action-to-receipt values 55,587–78,751 µs (median
  61,369 µs) and bus-managed values 56,898–67,959 µs (median 60,745 µs).
  `experiments/spiderweb-guard/LATENCY.json` has every row and binary hash.
  These small, noisy runs establish no speed advantage or hard latency bound.

**Correction to the earlier fit review:** current Spiderweb source buffers
lifecycle events, and its focused late-subscriber test passes. Older comments
describing an unbuffered lifecycle startup race are stale. Named callbacks
still have no replay and raw publisher success still does not acknowledge a
listener; the bridge explicitly registers and checks its subscriber.

This is a local source integration and generated-file verification. The
installed `/opt/ram-monitor` is older. The guard still protects only one
explicitly armed local 160-byte file, after a write, and is not a physical
Rowhammer prevention claim.
