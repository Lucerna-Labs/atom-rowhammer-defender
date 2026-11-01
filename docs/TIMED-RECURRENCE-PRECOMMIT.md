# Timed recurrence composition — precommit, 2026-10-31

The frozen adversarial search falsified the address-only composition: only
500/2,048 fast sampled recurrence attacks ruptured and 40/123 deliberately slow
or benign-locality controls ruptured. Timestamps were present but unused.

Build one independent time-dissipative recurrence stack. Its intrinsic observer
remembers the last timestamp of each full fingerprint and can perturb an
immutable swarm only when recurrence occurs before a fixed dissipation horizon.
It has its own state, owned copies, and receipt. It receives no attack/control
label and does not read, vote with, route to, or confirm another stack.

## Frozen construction comparison

Evaluate the candidate alone on the 2,048 fast adversarial traces and 123 timed
controls. Sweep the plain dissipation horizon before source integration. Then
freeze the best mechanically valid plain horizon and compare:

1. no chaos;
2. centered logistic-map horizon perturbation on every observation;
3. centered folded-Henon perturbation on every observation;
4. both perturbations composed at the same pre-recurrence position.

The plain unit is the bar. A chaos-bearing arm must strictly increase fast-attack
rupture coverage while retaining zero timed-control ruptures, deterministic
replay, finite bounded state, and monotonic timestamps. Plain wins ties. Preserve
all rejected arms.

## Integration gate

For timestamped observations, existing address-count channels may perturb their
swarms only on a locally hot recurrence; untimed replay retains the prior frozen
behavior. The timed recurrence stack also emits independently. Required:

- 2,048/2,048 adversarial attack traces rupture;
- 0/123 timed controls rupture;
- prior 10,752-case untimed corpus remains 8,704/8,704 attacks and 0/2,048 controls;
- full workspace tests and exact replay pass.

This remains a sampled-observation model. The selected horizon is not host DRAM
calibration or physical bit-flip proof.
