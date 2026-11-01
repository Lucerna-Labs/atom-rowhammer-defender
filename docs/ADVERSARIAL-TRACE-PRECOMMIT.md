# Adversarial trace search — precommit, 2026-10-31

Purpose: try to falsify the current immutable Rowhammer response composition
outside the named-family corpus. The evaluator is external. Runtime receives
only numeric observations and never receives a family, label, expected result,
route, or mitigation policy.

## Frozen search

1. Sweep periodic recurrence periods 1 through 512 across 1,024, 4,096, and
   8,192 raw observations, eight address translations, deterministic phase
   drift, 0/10/25/40 percent observation loss, and 0/5/10/20 percent unrelated
   observations.
2. Generate burst duty cycles, alternating epochs, nonuniform frequency/phase,
   period switching, shuffled recurrence, and sparse aggressor-plus-distractor
   traces.
3. Run matched benign-locality proxies: sequential and strided streams, random
   high-cardinality access, shuffled equal-histogram access, tiled scans,
   pointer-chase, Zipf locality over several working-set sizes, and repeated
   read-mostly regions at deliberately slower synthetic timestamps.
4. Repeat every retained witness exactly and compare serialized results.

The external risk oracle is separate from the app: a trace is an attack proxy
only when a synthetic refresh-epoch disturbance accumulator reaches its frozen
budget. Controls are required to stay below that budget. The oracle result is
never supplied to the app.

## Gate and falsifier

- every oracle-positive trace must produce at least one emergent rupture;
- every oracle-negative control must remain intact;
- exact replay must match;
- no panic or malformed receipt;
- labels remain external and immutable swarms remain independent.

Any miss or control rupture falsifies 100 percent effectiveness on this search.
Preserve the witness before changing composition. Do not retune the oracle or
remove a hard control after seeing results.

This is still a synthetic sampled-observation search. It cannot prove physical
ACT/PRE/REF coverage, DRAM mapping, bit-flip prevention, or universal efficacy.
