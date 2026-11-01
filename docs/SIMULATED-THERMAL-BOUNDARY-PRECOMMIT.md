# Simulated thermal-boundary adversarial search — precommit, 2026-10-31

Simulation only. This experiment uses generated numeric observations and never
opens, reads, or depends on CPU/PMU rings or live hardware counters.

The previous timed corpus separated fast sustained recurrence from deliberately
slow locality, but it did not test short fast bursts that stop before modeled
damage or repeated bursts separated by full cooling intervals. A stateful swarm
could accumulate those bursts across idle gaps and rupture even when the external
thermal model stays subcritical.

## Independent external oracle

For each generated trace, the evaluator keeps a mathematical recurrence-energy
state that is not visible to the app:

- an observation contributes one unit only when its fingerprint has recurred;
- recurrence energy decays exponentially with a fixed 500-microsecond time
  constant;
- danger begins at 128 accumulated recurrence units;
- a 5-millisecond idle interval is a cooling boundary;
- labels and oracle state remain outside the runtime.

## Frozen corpus

Attack proxies: sustained one- through 512-site recurrence, nonuniform and
period-switching recurrence, burst duty cycles whose accumulated energy crosses
128, address translations, time jitter, and deterministic observation loss/noise.

Controls: single subcritical bursts of 1 through 127 recurrence units, repeated
subcritical bursts separated by 5-millisecond cooling, fast nonrepeating streams,
slow recurrence, shuffled high-cardinality locality, and exact boundary cases at
127/128 oracle energy.

Required result: every oracle-positive trace ruptures; every oracle-negative
trace remains intact; exact replay matches; no runtime label, route, vote, or CPU
ring is introduced. Any failure is preserved before composition changes.

This oracle is a declared synthetic model, not DRAM calibration or physical
bit-flip evidence.
