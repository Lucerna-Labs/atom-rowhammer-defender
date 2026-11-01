# Timed evidence correction — 2026-10-31

Baseline80acf99 remains installed. The preceding160-case simulation measured
address-pattern response without validating physical danger. Keep those
receipts; their attack/benign labels are not a physical ground truth.

Fix ingestion, leaving the vendored Atom swarm intact: retain complete shifted
address identity in the concentration window; namespace virtual/instruction
addresses by PID and address kind; use the existing limited projection only
for swarm class selection. Capture kernel PERF_SAMPLE_TIME and explicit
instruction/data/synthetic provenance. Preserve timestamps, without pretending
that sequential per-CPU draining is a globally time-ordered ACT stream.

Predictions: unique16KiB-spaced addresses cease to concentrate; the eight alias
controls emit/trip0. Same-address samples from different PIDs or address kinds
do not merge. Existing concentrated replay remains responsive, and sample
timestamps/types parse through wrapped buffers. UI/CLI identify evidence as
sampled/proxy and confirm no physical ACT evidence. An identical benign/attack
hotspot remains indistinguishable from addresses alone.

Correct the simulation with an explicit synthetic bank/row mapping, row-open
state, cache hits, timestamps, periodic per-victim refresh, and configurable
disturbance budget. Only opening a different row or reopening a precharged row
adds modeled neighbor disturbance; reads hitting the open row/cache do not.
Independent grouped per-refresh activation accounting must agree with the
stateful model. Parameters are illustrative model inputs, not measured hardware
thresholds. Test fast/slow versions of the same address trace and identical
physical behavior under different intent labels; expect the address-only
detector to remain unable to separate some of these pairs. No threshold tuning
to the old fixture labels; report all remaining misses/false alerts.

Freeze test cases before implementation: alternate rows1/3 around victim2,
open-row repeated reads, cache hits, 8- and16-row cycling, 8-activation bursts
with intervening refresh, unique alias-stride accesses. Durations1ms/64ms,
disturbance budgets32/128, fast10us/slow10ms steps. Sweep these deliberately
synthetic profiles; do not claim any is this PC's physical DRAM. The same
observed slow/fast sequence can get the same detector output while modeled
danger differs. A live sample workload probe may measure IP/data provenance
and timing, never bit flips. Do not deploy without the existing exact-command
approval mechanism and successful runtime checks.

Input integrity precedes optimization: missing upper address bits and time
cannot be restored by chaos perturbation. No Atom mechanism is replaced.
