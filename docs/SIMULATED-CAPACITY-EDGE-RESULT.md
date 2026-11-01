# Simulated recurrence-capacity result — 2026-11-01

Simulation only. No CPU, PMU, or hardware-ring input.

The exact recurrence map passed periods through 65,536 but missed all three
period-131,072 cases because identities were evicted before reuse. A separate
independent bounded sketch thermal stack was added for that edge:

- three fixed 2^22-bit projections, about 1.5 MiB total;
- fixed 262,144-observation epoch cap;
- 500-microsecond heat decay and milestone lead law;
- full-cooling renewal;
- its own unchanged immutable swarm, state, owned copies, and receipt;
- no family label, route, vote, or sibling result.

Construction comparison over 30 attacks and 587 controls:

- plain: 30/30 attacks, 0/587 controls;
- folded-Henon thermal perturbation: ties plain, rejected because plain wins;
- logistic-index perturbation: 0/30 attacks, 0/587 controls, rejected;
- combined: 0/30 attacks, 0/587 controls, rejected.

Integrated results:

- periods 512 through 131,072: 30/30 attacks rupture;
- equal-length fast unique controls: 0/30 rupture;
- exact replay byte-identical, SHA-256
  `108218e19ffa6015e5c8fa1dc14ded2e69eb3985067fd5ee7c66267f64e34469`;
- random fuzz remains 5,000/5,000 positives and 0/5,000 negatives;
- exact-boundary latency remains 55/55;
- delivered thermal gate remains 33/33 positives and 0/156 negatives/controls;
- broader timed corpus remains 2,048/2,048 attacks and 0/123 controls;
- untimed known-technique corpus remains 8,704/8,704 attacks and 0/2,048
  controls;
- full Rust workspace: 79 passed, two existing privileged hardware tests ignored.

The owning application crate has 2,170 physical Rust source lines, below its
4,000-line limit; the new sketch module is 96 lines and adds no dependency.
This is bounded simulation evidence, not universal or physical effectiveness.
