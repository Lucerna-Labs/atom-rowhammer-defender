# Two-hop memory-stall companion v1 — specificity failure

Protocol `MEMORY-STALL-HOP-PRECOMMIT-2026-11-14.md` was frozen at
`1e4d9a4`; qutrit two-crate source at `d322f85`; evaluator at
`8632025`. The immutable native stack used three-site coherent serial
transport, source reset, information, binding, habituation, mechanics
and heat. It received only unit fractions and ran classically on the CPU.

The complete matrix contains3,562 trajectories ×96 ticks =341,952
new native ticks, with separate incumbent replay on the same heldout
cases. All finite, trace, PSD principal-minor/determinant, capacity
and unit-state checks passed. All137 A/full/plain traces repeated
exactly; all128 heldout plain traces matched debug/release byte for byte.
The compressed per-tick output `experiments/memory-stall-hop/raw.csv.gz`
has SHA-256 `b7b41146b00e3be0a06099603c9f2764515233f0b73075ee0be07c7bdfb2ad63`;
summary `matrix.csv` has SHA-256
`5c0f72c3437f806288b978695bf0783864b02e0cf9d4179a0987833b72a440b2`.
Full locked offline workspace tests and the portable-only guard passed.

| Arm on fresh32-seed holdout | L8 short controls /32 | L16 target /32 | L32 long /32 | Quiet /32 |
| --- | ---: | ---: | ---: | ---: |
| New A/full/plain | 32 | 32 | 32 | 0 |
| New B/full/plain | 32 | 32 | 32 | 0 |
| New A/no-quantum or diffusion | 0 | 0 | 0 | 0 |
| Existing fixed two-level stack | 0 | 17 | 32 | — |
| Classical 16-tick rolling mean or run length | 0 | 32 | 32 | 0 |

All five chaos arms tied the full plain response counts, including the
32/32 L8 false responses; no chaos arm was promoted. The candidate
responded to all15 incumbent L16 misses, giving external OR coverage
32/32, but the OR also inherits **32/32 L8 false responses**. This fails
the frozen <=2/32 short-control gate and the useful complement gate.
The simple external classical rules satisfy the target and short-control
separation, so no algorithmic or quantum advantage is claimed.

The failure is mechanistic, not a missing source or serialization fault.
With seed701's first8-tick high block, third-site occupation p2 was
0.0413 at tick23 and remained0.0129 after the following8 low ticks;
off-diagonal link coherence stayed0.0316. Chemical binding *rose*
from0.0180 to0.0345 during that low interval because retained p2 still
associated faster than dissociation. After the second high block at
tick39, p2 reached0.1191 and E0.000067; after the third at tick55,
E0.000325 exceeded the frozen0.0001 capacity. Across heldout L8
controls, first crossings ranged ticks38..55 and peak E ranged
0.000149–0.001534. The source reset and binding kinetics carried state
across low intervals, so four short blocks accumulated as a long event.
Order, stage, normalization and the selected chaos survivors were
measured; none fixed that false response. The complete quantum-derived
base was load-bearing for both desired and undesired responses.

**Disposition:** failed candidate, preserved in source history and raw
artifacts; no app or library admission. A separate frozen correction may
strengthen low-state relaxation and make binding require ongoing input,
then use entirely fresh seeds. The incumbent source and app bridge
remain unchanged. No malicious-pressure classifier, live action or
physical quantum claim follows.
