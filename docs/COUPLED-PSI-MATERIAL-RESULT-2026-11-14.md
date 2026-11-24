# Coupled memory/I/O material — bounded numerical response

The protocol was frozen at `29ae406`, two-crate source at `3be54fe`,
and native evaluator at `7fb30ea`. The engine accepts only two finite
unit fractions per tick and returns four-state density/material values.
No resource name, process identity, attack label, policy, collection or
mitigation enters the native crates. Quantum-derived density math runs
on a classical CPU.

The matrix has4,438 trajectories ×96 ticks =426,048 native ticks.
All finite, trace1, real-symmetry, PSD-principal-minor, bounded-state
and material checks passed. All231 A/full/plain traces repeated
byte-identically; all224 heldout plain traces matched debug/release
byte for byte. The full locked offline workspace tests and portable-only
guard passed. Raw per-tick `experiments/psi-coupling/raw.csv.gz`
SHA-256: `fc7b98d18eb071ba77ff15469b555240612759719e6e88761c7fe0e66030d372`.
Summary `matrix.csv` SHA-256:
`dde312c259ee2188b851b53eba855e99d68665c57a7b79bf47d14802e7cecd67`.

| Arm on fresh32-seed holdout | Full simultaneous /32 | Anti-phase /32 | Half-overlap /32 | Memory-only /32 | I/O-only /32 | Quiet /32 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A/full/plain | 32 | 0 | 0 | 0 | 0 | 0 |
| B/full/plain | 32 | 0 | 0 | 0 | 0 | 0 |
| A/diagonal no-quantum | 0 | 0 | 0 | 0 | 0 | 0 |
| A/classical population diffusion | 0 | 0 | 0 | 0 | 0 | 0 |
| A/no chemistry or no mechanics | 0 | 0 | 0 | 0 | 0 | 0 |
| External16-tick simultaneous-run | 32 | 0 | 32 | 0 | 0 | 0 |

All32 paired complete-versus-anti-phase mean energies were positive.
Full positive peak material ranged0.00621–0.14725 (mean0.03930),
with first crossings ticks50..63; anti-phase peak was at most
1.95e-11. Half-overlap peak was at most0.004490, just below the
frozen0.005 capacity. The no-quantum and matched diffusion populations
had mean positive peaks5.53e-7 and5.03e-7 respectively. Coherence is
load-bearing relative to those declared population controls. Removing
information, cognition or physics changed all32 continuous positive
trajectories; no-cognition also responded to2/32 half-overlap traces.
Chemistry and mechanics removal zeroed the material product by design.
Both order and all five chaos arms tied the scored response counts, so
A/full/plain/seed0 remains selected and no chaos arm was promoted.

The per-channel value multisets and final samples were exactly matched
between positive, anti-phase and half-overlap profiles; only timing
across the two inputs differed. A legitimate simultaneous-high witness
was byte-identical to the positive and responded32/32, showing the
input cannot identify hostility. The external classical rolling-product
and simultaneous-run rules each found32/32 positives and0/32
anti-phase/single-resource/quiet controls, while also responding to
32/32 half-overlap. There is no exclusive quantum algorithmic or
speed advantage.

**Admitted scope:** an intrinsic numeric response to complete32-tick
simultaneous high memory/I/O stalls on the frozen synthetic .70–.90
fractions, quiet on the declared anti-phase and shorter-overlap
profiles. It is a separate two-input function from the one-input
memory-stall stacks. It does not establish swap, attacker cause, live
system protection, general workload specificity, or physical quantum
behavior. Any app bridge must keep a strict uploaded-trace boundary
and report state without a verdict.
