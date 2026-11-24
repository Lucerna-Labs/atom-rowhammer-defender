# Adversarial timed recurrence result — 2026-10-31

> **Correction — 2026-11-02.** The 50,000-nanosecond horizon and the 2,048/2,048
> and 0/123 counts below describe the frozen external evaluator and its corpus.
> They do not describe the shipped `ram-monitor` crate, and must not be read as
> a coverage statement for it. The shipped gate is the thermal recurrence model
> in `crates/ram-monitor/src/lib.rs`: one unit of heat per recurrence of a
> fingerprint still inside a 4,000,000 ns retention window (`lib.rs:274`,
> `lib.rs:300-302`), exponential decay with a 500,000 ns time constant
> (`lib.rs:294`), and danger at 128 units (`lib.rs:327`). Its steady state is
> `H = 1 / (1 - e^(-dt/tau))`, so 128 units require a mean access spacing of
> `dt <= 3,921.6 ns`. Measured with the shipped CLI on synthetic traces: a
> constant address recurring every 3,921 ns ruptures after 1,125 samples; at
> 4,000 ns it does not rupture in 4,096 samples; a balanced 4,096-line scan at
> 50 ns spacing (204,800 ns per-line recurrence) ruptures at 4,225 samples.
> The 50,000 ns horizon is therefore not the shipped rule: it is neither
> sufficient (a 4,000 ns recurrence is missed) nor necessary (a 204,800 ns
> recurrence ruptures). The recorded measurements below are unchanged; this
> correction supersedes their interpretation as shipped behaviour. Regression
> pins live in `crates/ram-monitor/tests/immutable_secondary_gates.rs`
> (`shipped_band_constant_recurrence`,
> `fast_balanced_scan_ruptures_known_limitation`); the required-but-unmet
> behaviour is encoded, ignored, in
> `crates/ram-monitor/tests/documented_horizon_gates.rs`.

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

> Correction, 2026-11-02: this sentence describes the external evaluator only.
> The shipped crate is a repeat-density heat integrator with a 4 ms retention
> window and a ~3.92 us mean-spacing boundary; see the correction at the top.

Plain horizon sweep:

- 25,000 ns: 2,012/2,048 attacks, 0/123 controls;
- 50,000 through 950,000 ns: 2,048/2,048 attacks, 0/123 controls.

The lowest tied horizon, 50,000 ns, was retained. At that horizon the plain,

> Correction, 2026-11-02: "retained" refers to the evaluator's sweep setting,
> not to a constant in the shipped crate. No 50,000 ns value exists in
> `crates/ram-monitor/src/`.
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
- full Rust workspace: 75 passed at the time of this run, two ignored
  by their existing gates.

This closes the frozen sampled-observation counterexamples. It does not calibrate
the 50-microsecond horizon to this host's DRAM, prove live sample completeness,
exercise ACT/PRE/REF or physical row mapping, or prove bit-flip prevention.
