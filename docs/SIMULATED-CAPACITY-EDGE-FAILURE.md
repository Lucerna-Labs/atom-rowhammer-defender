# Simulated recurrence-capacity failure — 2026-11-01

Simulation only; no CPU or hardware-ring input.

The frozen capacity sweep ran ten recurrence periods under three timing profiles
with equal-length nonrepeating controls:

- periods 512 through 65,536: 27/27 attacks ruptured;
- period 131,072: 0/3 attacks ruptured;
- unique-control ruptures: 0/30.

The exact fingerprint map is capped at 65,536 entries. At period 131,072 each
identity is evicted before recurrence, so the thermal stack never receives a
recurrence unit even though the external delivered-stream oracle eventually
crosses its danger boundary.

Preserve the exact bounded stack. Test a separate independent fixed-size sketch
thermal stack for the high-cardinality edge. It must have its own state and
immutable swarm, clear on cooling and bounded epochs, pass all three period-
131,072 attacks, and keep all 30 unique controls silent. Compare plain,
logistic-index, folded-Henon thermal, and combined chaos arms; plain wins ties.
No labels, votes, routes, unbounded map, or CPU rings.
