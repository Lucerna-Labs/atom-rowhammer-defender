# Known-technique black-box stress result — 2026-10-31

The frozen external evaluator ran 10,752 deterministic address-order traces:
8,704 modeled attack traces and 2,048 near-controls. Technique names and expected
results existed only in the evaluator. The runtime received numeric addresses.

The prior integrated composition ruptured 7,552/8,704 modeled attacks and
0/2,048 controls. Misses were isolated to observation-loss/noise variants of
16-, 32-, and 59-sided recurrence.

A third independent immutable swarm was composed behind a fixed full-fingerprint
recurrence observer with a 768-observation horizon and count threshold 11. It has
its own owned copies, state, and receipts; it does not vote, route, identify a
technique, or read another stack.

Edge-of-chaos construction check:

- initial 512/10 plain arm: 1,408/1,536 focused attacks, 0/2,048 controls;
- initial chaos-on arm: 1,536/1,536 attacks, but 16/2,048 controls, rejected;
- longer-horizon 768/11 plain arm: 1,536/1,536 attacks, 0/2,048 controls;
- longer-horizon chaos-on arm tied coverage and controls, so plain won the tie.

Final integrated-source result:

- 10,752/10,752 cases completed without evaluator error;
- 8,704/8,704 modeled attack traces produced an emergent rupture;
- 0/2,048 controls produced a rupture;
- all 17 modeled attack families passed 512/512;
- all four control families stayed silent 512/512;
- an exact second replay produced byte-identical JSON, SHA-256
  `b5dc4e3fb4be91401f0b7c92394aef1dbeb8c229ebeee65fea63e1f723f12677`;
- the full Rust workspace passed 72 tests, with two explicitly privileged live
  hardware tests ignored by their existing gates.

First rupture ranged from 6 observations for concentrated traces to 610 for the
hardest 59-sided loss/noise witness. Address-order success remains proxy evidence,
not physical validation of ACT/PRE/REF timing, DRAM row mapping, row-open time,
refresh synchronization, bank topology, or bit-flip prevention.
