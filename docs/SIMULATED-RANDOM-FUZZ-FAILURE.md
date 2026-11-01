# Randomized simulation fuzz failure — 2026-10-31

The frozen deterministic fuzz gate completed 10,000 delivered traces:

- positives passing at or before danger: 3,774/5,000;
- positive misses: 1,226/5,000;
- negative ruptures: 0/5,000;
- translation mismatches: 0/5,000 paired translations.

First failures had visible oracle peaks just above 128 and occurred identically
under address translation. Diagnosis: the integrated thermal state expires a
fingerprint after one millisecond, but the declared external recurrence-energy
model retains delivered fingerprint history across longer intervals while heat
itself decays continuously. Random periods and jitter can recur after one
millisecond, so the app discards an input relation the oracle still owns.

This is a composition/input-retention mismatch, not weak chaos. Preserve the
failing witnesses. Test bounded recurrence-memory horizons of 1, 2, 4, 8, 16,
32, and 64 milliseconds on the frozen positive failures, then rerun all 10,000
cases and prior controls with the smallest passing horizon. Keep the 65,536-entry
hard cap. No CPU ring or label may be added.
