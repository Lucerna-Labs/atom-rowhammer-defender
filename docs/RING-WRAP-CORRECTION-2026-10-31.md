# Perf ring wrap correction — before implementation

Source review found an additional live coverage defect after the first repair
precommit: `record_at()` rejects any header or record that crosses the end of
the perf mmap ring. Its caller returns `None` without moving `data_tail`, so
all later calls can remain stuck at that record while the kernel continues to
write. The live service's flat `frames=3024` despite later changes in
`ip_fallback` is consistent with a stall but does **not** prove this was its
cause. The old `lost=351` is still untrusted because its parser reads event
id as loss count.

Fix the ring reader to copy the small fixed header and only the required
sample/loss fields across the circular boundary. Keep `PERF_RECORD_LOST`
and `PERF_RECORD_LOST_SAMPLES` layouts distinct. Require the entire record
to be within the published `data_head` before advancing `data_tail`; reject
malformed lengths and log the loss of trust. Prediction: synthetic wrapped
headers and wrapped sample/loss bodies parse exactly, the tail advances,
and the existing replay/perf controls remain green. The full live gate still
requires deploying the tested binary and observing a new service invocation.
