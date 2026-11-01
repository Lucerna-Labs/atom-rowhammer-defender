# Stateful uploaded-sequence gate — precommit, 2026-11-01

Simulation only; no CPU or hardware-ring input.

Generate 1,000 positive uploaded traces containing long safe prefixes,
near-threshold plateaus, multiple fully cooled epochs, a final sustained attack,
and address translations. Generate 1,000 matched all-safe multi-epoch controls.
Each trace is processed by one persistent monitor instance, exactly as an upload.

Required:

- no positive trace halts inside its safe prefix;
- all 1,000 positives halt at or before their first delivered danger;
- none of 1,000 all-safe sequences halt;
- cooling and bounded-sketch renewal cannot leak progress across epochs;
- exact replay and translation invariance;
- installed/file parser and no-CPU boundaries remain unchanged.

Preserve any cross-phase counterexample before repair. Passing remains bounded
simulation evidence.
