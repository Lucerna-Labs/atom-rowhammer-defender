# Simulated rupture-halt gate — precommit, 2026-11-01

The portable app currently records a rupture but continues consuming the
uploaded trace. That demonstrates simulated response, not prevention.

Add a simulation-only halt boundary: the first receipt from any independent
swarm stops consumption of the current uploaded/generated trace. The application
does not interpret the receipt, identify a technique, wait for another stack, or
vote. Reset/reload explicitly clears the halt.

Requirements:

- CLI trace execution reports `halted=true` and consumes no observation after
  the first rupture;
- GUI/controller snapshot exposes HALTED and stops advancing sample count;
- all 55 exact-boundary positives halt at or before first danger;
- 5,000 random positives halt by danger;
- 5,000 random negatives, 57 subcritical controls, 123 timed controls, and 30
  capacity controls never halt;
- malformed input still fails closed;
- no CPU/hardware ring, label, classifier, vote, or external connection.

This is prevention only inside the uploaded mathematical simulation. It is not
physical mitigation.
