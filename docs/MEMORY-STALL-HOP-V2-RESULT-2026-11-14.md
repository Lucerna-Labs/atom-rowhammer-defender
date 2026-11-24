# Two-hop pressure companion v2 — bounded complementary response

The correction protocol was frozen at `d8b3168`, source at `068326e`,
and fresh evaluator at `98bb0e3`. The v1 failure remains in
`MEMORY-STALL-HOP-V1-RESULT-2026-11-14.md` and its original raw output.
V2 changed only the convex low-state reset coefficient `.16→.30` and
made chemical association proportional to the current scalar input.
Qutrit transport, fixed capacity0.0001, stack ordering, chaos arms,
other domains and the separate two-level incumbent stayed fixed.

The fresh matrix contains3,562 new trajectories ×96 ticks =341,952
native ticks, plus matched old-v1 and incumbent runs. All finite,
trace1, PSD principal-minor/determinant and bounded-state checks
passed. All137 A/full/plain traces repeated byte-identically; all128
heldout plain traces matched debug/release byte for byte. Full locked
offline workspace tests and portable-only guard passed. Raw per-tick
`experiments/memory-stall-hop-v2/raw.csv.gz` SHA-256:
`7acfe8d8eaa1067e10dfd9401d1f192661d8804d5abd0652fd63928330b7f322`.
Summary `matrix.csv` SHA-256:
`3837647acfe0af3ea7355b8ade18a62c501acba1d2483ea848b1fa9e991e37c3`.
The old-v1 binary used for same-trace comparison had its frozen
SHA-256 `3691531b34cb803bf4b2e40bb0389b09a21c6d604b76a97d65aea91e9abaefd5`.

| Full plain on fresh32-seed holdout | L8 short controls /32 | L16 target /32 | L32 long /32 | Quiet /32 |
| --- | ---: | ---: | ---: | ---: |
| New two-hop v2, order A | 0 | 32 | 32 | 0 |
| New two-hop v2, order B | 0 | 32 | 32 | 0 |
| Two-hop v1, order A | 32 | 32 | 32 | — |
| Fixed two-level incumbent, order A | 0 | 19 | 32 | — |
| External rolling mean or run-length rule | 0 | 32 | 32 | 0 |

The v2 component responded to all13 L16 profiles the incumbent missed.
Their external OR reaches32/32 L16 and keeps0/32 L8 false responses.
This meets the frozen bounded complement gate. All32 paired L16 mean
energies exceeded L8. V2 L16 first crossings were ticks27..31, before
the later low interval; L8 peak energies stayed at or below0.00001147,
below capacity0.0001. L16 peak energies ranged0.000175–0.005502.

The coherent qutrit base is load-bearing against both matched diagonal
population and diffusion controls: each had0/32 L16 responses, versus
32/32 full. Those two population controls are near-identical for this
input and should not be mistaken for independent evidence of quantum
superiority. No-chemistry and no-mechanics removed the material response
by its product law. Removing information, cognition or physics changed
all32 continuous L16 trajectories but not their crossing counts.
All five chaos arms and B order tied plain crossing counts; none was
promoted. A benign scheduled L16 witness with identical input produced
the identical crossing, so the engine cannot infer hostile intent.

**Admitted scope:** a second, independent qutrit material stack that
numerically responds to repeated16-tick high memory-stall episodes in
the specified96-tick synthetic profiles, complementing the existing
two-level long-dwell stack. It is one corrected qutrit family, not a
new family for v1, v2, each order or chaos setting. A simple ordinary
run-length/rolling-mean rule matches the scored separation, so no
exclusive algorithmic, physical quantum or speedup claim follows.
The input fractions .78–.90 are severe synthetic pressure; realistic
workload false-alarm rates, live collection, attacker attribution and
mitigation remain untested. App wiring needs an independently frozen
bridge with separate numeric receipts and no in-engine OR policy.
