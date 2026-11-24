# Automatic file restoration: generated-file result

Scope: the user selected an automatic response that restores trusted bytes and
preserves the changed file. This result covers one explicitly armed, local
160-byte regular file. It does not establish physical Rowhammer mitigation or
protection of arbitrary files. The existing installed `/opt/ram-monitor` was
not replaced during these experiments.

## Operational path

`--source integrity-guard` and the GUI's **ARM AUTOMATIC RESTORE** capture a
trusted reference once, watch the selected name with Linux inotify, and run a
bounded periodic fallback recheck. An exact byte or object-identity difference
triggers response. The same opened file supplies bytes and identity for a
response snapshot. Before action, the guard writes an exclusive mode-0600
`.changed` copy and a `.meta` sidecar, syncs both, reads back the copy, and
syncs the private evidence directory. A missing path gets a synced `.absent`
record. Failure to preserve stops action.

A selected modify/create event remains **pending** until the corresponding
close-write; the guard does not treat the empty middle of a truncate/write
sequence as a completed file. Pre-action snapshot races retry up to eight
times with 20 ms event waits, preserving each version actually captured. It
never retries after mutation or evidence failure. A writer that never closes
leaves an unresolved pending state.

The pinned `atom-cognitive-engine` `observe_stack` and `query_stack` select
between an in-place rewrite and an atomic exchange. Only verified successful
repairs enter session memory. Blank or contradicted memory selects atomic;
an object change forces atomic regardless of the claim. Atomic exchange uses
`renameat2(RENAME_EXCHANGE)` and retains the displaced file object through a
hardlink in the evidence directory. Both methods verify exact trusted bytes
after action; object identity is rearmed only after that check. The cognitive
engine cannot suppress a hard integrity trigger. Every verified action also
gets an exclusive synced `.result` record with the chosen/executed methods,
activated cognitive trace IDs, response hash, object identities, and timing.

## Measured generated cases

The committed `experiments/cognitive-auto-restore/probe.py` drove the actual
source-built CLI on generated ext4 files. `RESULT.json` records the binary
hash, output, per-case evidence hashes and action-to-receipt times. Six
restorations were verified: two one-byte edits in a nonwritable parent, a
same-content object replacement, a one-byte edit with a read-only file mode,
deletion, and a deliberately slow 161-byte streaming write. The slow write
reported two pending checks and no action before close, then retained the
complete 161-byte changed copy and restored the trusted path. The first nonwritable-parent event
selected atomic on blank memory, failed before mutation, and used in-place;
the second selected in-place directly from verified session memory. The
same-content replacement used the hard object gate and retained its displaced
inode. The 161-byte changed object was preserved in full. Evidence-directory
write failure left the changed file untouched and exited 4. An unrelated
sibling event caused no action. The reference stayed byte-identical.
All six repairs had durable `.result` records; the preservation-failure and
unrelated-event cases had none.

The six action-to-CLI-receipt observations were 36–91 ms in the recorded run;
the restoration work itself reported 31–38 ms. These are observations on one host,
not a latency guarantee. Earlier failed probe attempts are preserved as
`FIRST-ATTEMPT.json`, `SECOND-ATTEMPT.json`,
`RACE-FIRST-ATTEMPT.json`, and `THIRD-ATTEMPT.json`; the first two were
harness assertions/fixture timing, the race fixture initially armed an
already altered file, and the third captured the real incomplete-write race
that led to defer-until-close behavior. The corrected
atomic-exchange race test shows a second replacement inserted after preflight
was retained in `.original-object` while the selected path became trusted;
the guard reported the changed target as a race instead of learning from it.

The GUI was exercised under Xvfb with generated files. It visibly armed,
restored a one-byte edit, showed evidence and method, then disarmed; a second
edit after disarming remained changed. `gui-restored.png` and
`gui-disarmed.png` are the visual records. This is an isolated virtual-display
check, not proof that the installed desktop launcher runs the new source.

The root `cargo test --workspace --locked --offline --quiet` passed with 152
test passes in its output; `scripts/check_portable_only.py` passed.

## Method-selection gate

The frozen gate is `docs/COGNITIVE-SELECTOR-GATE-PRECOMMIT-2026-11-14.md`.
`experiments/cognitive-selector/HELDOUT.csv` contains 32 held-out cases across
two permission contexts. The cognitive selector chose the expected method in
32/32; fixed atomic-first would incur 16 first-attempt failures; a plain
permission-bit rule also chose 32/32. Blank memory, object override and
opposite-origin conflict took the specified safe fallbacks. A short 3-seed,
18-query chaos arm tied plain 18/18 with zero strictly better successors, so
plain was retained. This gate shows a measured session-learning behavior, not
unique predictive accuracy over the simple rule.

## Limits before broader deployment

- Inotify reports after a change. It cannot veto the write, identify its
  originator, or guarantee that a transient edit will be seen. Periodic
  rechecks cover missed wakes only while the altered state persists.
- A live concurrent writer can race a post-event response. The atomic
  exchange preserves a displaced object and reports a mismatch. In-place
  rewrite is needed for the tested nonwritable-parent case, but an
  uncooperative writer may alter that same object after the prewrite check.
  Do not claim lossless preservation of every concurrent version.
- Trusted-reference authenticity and evidence-directory protection against
  another process with the same user privileges are outside this guard.
  Session learning resets on disarm/restart. The test covers one file and a
  1 MiB maximum changed-file snapshot, not general host protection.
- Spiderweb Bus Next can carry post-verification telemetry and supervise
  independent sensors. Its publisher succeeds with zero subscribers, and its
  named subscribers have no replay. **Correction:** the earlier lifecycle-race
  claim here came from stale comments; current source buffers lifecycle and
  the focused replay test passes. The check/preserve/restore/verify transaction
  remains in one worker; no required action depended on a bus message in this
  initial build.

Source: frozen gate commits `2c518cc`, `3f2ef89`, `36827ad`; source-built CLI
and GUI probes; root workspace test log under
`target/cognitive-source/workspace-test.log`; exact source in this branch.
