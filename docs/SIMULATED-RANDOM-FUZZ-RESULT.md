# Randomized simulation fuzz result — 2026-11-01

Simulation only. No CPU, PMU, or hardware-ring input.

The first frozen 10,000-case run passed 3,774/5,000 positives, missed
1,226/5,000 positives, ruptured 0/5,000 negatives, and had zero translation
mismatches. The one-millisecond fingerprint-retention horizon discarded recurrence
relations still present in the declared delivered-stream oracle.

Frozen positive retention sweep:

- 1 ms: 3,774/5,000;
- 2 ms: 4,884/5,000;
- 4 ms: 5,000/5,000;
- 8, 16, 32, and 64 ms: each 5,000/5,000.

Four milliseconds is the smallest passing horizon and is retained. The
65,536-entry hard cap and heat-decay/cooling mechanics remain unchanged.

Final full gate:

- positives rupturing at or before first danger: 5,000/5,000;
- negative ruptures: 0/5,000;
- translated-pair mismatches: 0/5,000;
- exact replay byte-identical;
- result SHA-256
  `7e02cbce4c93159aba90cd2e68221c554f1d429cffd9a107f9b182b7ae6cd75c`.

The grammar covered periodic/nonuniform recurrence, changing periods, jittered
timing, cooling epochs, near-threshold plateaus, high-cardinality streams,
address translation, and mixed novel/recurring fingerprints. This is bounded
simulation evidence, not universal or physical effectiveness.
