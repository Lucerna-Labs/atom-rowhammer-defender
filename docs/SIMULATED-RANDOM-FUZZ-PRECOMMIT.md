# Randomized simulation fuzz gate — precommit, 2026-10-31

Simulation only; no CPU or hardware-ring input.

Run a deterministic generated corpus beyond the hand-authored families. Produce
5,000 delivered-stream thermal positives and 5,000 negatives from fixed seeds.
The grammar includes periodic and nonuniform recurrence, changing periods,
randomized inter-observation intervals, burst/cooling epochs, near-threshold
plateaus, address translations, high-cardinality streams, and mixed recurring and
novel fingerprints. Trace lengths range from 32 through 4,096 observations.

The frozen external oracle remains the declared recurrence-energy model:
500-microsecond exponential decay, one unit per delivered fingerprint recurrence,
and danger at heat 128. Labels remain outside runtime.

Required:

- 5,000/5,000 positives rupture at or before the first dangerous observation;
- 0/5,000 negatives rupture;
- exact replay and translation invariance;
- no panic, label, route, vote, CPU ring, or hidden-event expectation.

Preserve the first counterexample of every failure class before changing
composition. Passing this gate remains bounded simulation evidence, not universal
or physical effectiveness.
