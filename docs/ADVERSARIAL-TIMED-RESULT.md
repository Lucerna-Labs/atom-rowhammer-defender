# Adversarial timed recurrence result — 2026-10-31

The frozen adversarial evaluator ran 2,048 fast attack traces: recurrence periods
1 through 512, each under clean, 40 percent loss plus 20 percent noise, 75 percent
loss plus 10 percent noise, and 90 percent loss plus 5 percent noise. It also ran
123 timed controls: slow recurrence, sequential, stride, permutation,
pointer-chase, Zipf locality, and tiled scans. Labels and the synthetic risk model
existed only in the evaluator.

## Falsification

The address-only integrated source ruptured 500/2,048 attack traces and 40/123
controls. It missed clean periods beginning at 77 and responded to eight of eleven
slow-period controls plus every Zipf control. Timestamps were supplied but ignored.

## Construction result

A new independent time-dissipative recurrence observer feeds its own unchanged
immutable swarm. It remembers only the most recent timestamp per full fingerprint
and emits when recurrence occurs within 50,000 nanoseconds. It has separate state,
owned copies, and receipts and reads no sibling result.

Plain horizon sweep:

- 25,000 ns: 2,012/2,048 attacks, 0/123 controls;
- 50,000 through 950,000 ns: 2,048/2,048 attacks, 0/123 controls.

The lowest tied horizon, 50,000 ns, was retained. At that horizon the plain,
centered logistic, centered folded-Henon, and combined chaos arms each produced
2,048/2,048 attacks, 0/123 controls, with the same first-rupture range 3 through
517 observations. Chaos did not strictly improve the plain bar and was not
promoted.

## Integrated result

For timestamped observations, the prior count observers may perturb their swarms
only on a locally hot recurrence; untimed replay retains the previous behavior.
The new timed swarm also emits independently.

- adversarial timed corpus: 2,048/2,048 attacks, 0/123 controls;
- exact second replay byte-identical, SHA-256
  `d351cd03abe5f0357e7f2c36aef7746a8100d6fbc591298fe70184f4311af987`;
- prior untimed corpus unchanged: 8,704/8,704 attacks, 0/2,048 controls, exact
  prior SHA-256
  `b5dc4e3fb4be91401f0b7c92394aef1dbeb8c229ebeee65fea63e1f723f12677`;
- timed state expires after the horizon and is hard-capped at 65,536 entries;
- full Rust workspace: 75 passed, two explicitly privileged hardware tests
  ignored by their existing gates.

This closes the frozen sampled-observation counterexamples. It does not calibrate
the 50-microsecond horizon to this host's DRAM, prove PMU sample completeness,
exercise ACT/PRE/REF or physical row mapping, or prove bit-flip prevention.
