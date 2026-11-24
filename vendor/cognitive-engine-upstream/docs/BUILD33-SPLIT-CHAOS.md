# BUILD-33 — the split chaos stage: the affine pieces (gain, rotation, translation) and their orders (precommitted 2026-09-27, before code)

Under vault Rules 1–7, project rules 0/0a/0b, Jesse's **D62** (a function may
be split across positions) and his **"Yes try"** (2026-09-27).

## THE RULE CONFLICT, SURFACED (not silently resolved)

The deployed hook payload and the canonical Obsidian notes **disagree**:

- **Deployed hook** (`~/.local/share/atom-agent-hooks/ATOM-RULES-CONTEXT.md`,
  modified **2026-09-27 09:44**): Rule 2 = *"Choose **one to three**
  mechanically justified candidate orders from intrinsic dependencies, state
  compatibility, and plausible interactions. Precommit the rationale, metrics,
  controls, and bounded execution count; test every declared candidate. **Do
  not default to random or exhaustive factorial sweeps.**"* — and Rule 5/7
  follow it ("Preselect mechanically justified chaos positions within Rule 2's
  one-to-three logical candidate orders… every possible placement is not
  required"; "Complexity does not require factorial enumeration").
- **Canonical notes on disk** (Rule 2 mtime 2026-09-25, Rules 5/7 mtime
  2026-09-26 23:10): the **original exhaustive text** — Rule 2: *"run **every unique permutation** of their stacking order; for `n` distinct primitives that
  means `n!` executions per controlled condition"*; Rule 5: *"execute every
  unique order"*; Rule 7: *"run every unique order of each declared finite
  primitive set"*.

**This build applies the hook's newer, bounded discipline** (it is the text
injected into every agent today, and the text the operator re-sent as
mandatory context), **and it is designed to be VALID UNDER BOTH READINGS**:
the declared finite set is only **three** pieces, so the *complete* order
space is `3! = 6` — cheap enough that the full permutation sweep **is** the
bounded candidate set. Nothing is claimed beyond the executed orders, and the
notes-vs-hook discrepancy is recorded here for the operator to settle (the
notes likely need updating to match the deployed hook).

## The split (D62 applied to BUILD-32's rejected whole function)

BUILD-32 treated the modulation as ONE function — a bounded scalar gain — and
measured it inert on Stack A (identical contamination 16 across arms). D62
says split a function into its genuine dynamical pieces and place each where
it excels. The mechanical split comes from the substrate's own structure:
**the elementary operations on a complex field are the affine pieces**, and
each interacts differently with the consumer's readout (`difference_vectors`
= the DFT **power** spectrum of the *summed* pattern, thresholded absolutely):

1. **Gain** — `state *= 1 + α(u − 0.5)` (declared α = 1.0, factors in
   [0.5, 1.5]). Magnitude only; measured inert at this consumer (BUILD-32:
   the residue dominates the absolute threshold, so *which* bins clear it
   never changed). Piece included as the recorded comparison.
2. **Rotation** — `state *= e^{iθ}`, `θ = π(2u − 1) ∈ (−π, π)` (declared
   convention). The consumer's readout is the power of the **sum** of the
   carried residue and the new contribution, `|r e^{iθ} + n|²`, whose cross
   term `2 Re(r e^{iθ} n̄)` is **explicitly phase-dependent** — so unlike the
   gain, this piece can move the number.
3. **Translation** — the carried pattern is rolled on the lattice (min-image,
   the engine's own torus convention) by `dx = ⌊N·u⌋`, `dy = ⌊N·u′⌋` with
   N = the lattice size. The residue's own magnitude spectrum is preserved
   under a shift, but the **relative** phases against the new contribution
   are not — the summed power moves. A third, distinct interaction point.

**Mechanisms (Rule 5's two, by intrinsic dynamics, already verified in
BUILD-32):** logistic `r = 4` and Hénon `(a, b) = (1.4, 0.3)` folded mod 1.
**Matched parameters:** each piece draws from its **own dedicated orbit**
(gain-orbit, phase-orbit, shift-x-orbit, shift-y-orbit), so the perturbations
are *identical across every order* and only the **order of application**
varies — attribution is clean.

## Positions and orders

The pieces are applied at the **END-of-run position** (the operator's
narrowing stands); the *position variable inside that stage* is the order of
application. The declared finite set is `{Gain, Rotation, Translation}` →
**3! = 6 unique orders, all executed** under each of the two mechanisms.
A macro-position variant (mid-stack, before the consumer readback) is named as
a **separate future experiment**, not silently merged here.

## Frozen metrics, controls, prediction, falsifier

- **Benefit metric (frozen, identical to BUILD-32 for direct comparability):**
  the **contamination total** — Σ over the five follow-on tasks of |carried
  support − fresh support|, an exact integer; the recorded baseline is **16**.
- **Hard invariants:** the first task's readout equals its fresh value, and
  every arm's verdict fields (closure, both sufficiency verdicts,
  derivability, judge, economy) equal the direct run's, field for field.
- **Controls (matched, same tasks, same pad lifecycle, same draws):**
  (a) **no chaos** (baseline 16); (b) **gain-only** (BUILD-32's arm — must
  reproduce 16, proving the setup is comparable); (c) each single piece alone
  (3 arms per mechanism) — the Rule-5 single-chaos controls that locate the
  contribution.
- **Execution count, declared and bounded:** 6 tasks × (6 orders × 2
  mechanisms + 3 single-piece × 2 mechanisms + 2 controls) = 6 × 20 = **120
  pad runs**, plus the invariant sweep. The resource reason for executing the
  full 6-order space (rather than a 3-candidate subset): it costs 120 short
  runs, it satisfies the notes' exhaustive reading as well as the hook's
  bounded one, and it removes any need to argue that a chosen subset was
  representative.
- **Prediction (precommitted):** at least one order produces a **measurable
  difference** from 16 — rotation and translation are not inert the way the
  gain was — and **no order strictly reduces** the contamination, because any
  perturbation adds structure to a residue whose readout is absolute-
  thresholded. Verdict therefore expected: **chaos rejected again, now with
  per-piece, per-order attribution.**
- **Falsifier:** any arm whose contamination is **strictly less than 16**
  with the invariants held ⇒ that piece/order is promoted as the first
  chaos-wiring candidate, and the sign-flipped control (θ → −θ, shift →
  −shift) runs next to test whether the effect is |u|-driven rather than
  dynamics-driven.

## Precommitted gates

**F1 (pass) — the pieces are distinct operations.** Each piece changes the
state in its own way and only its own way: gain changes magnitudes and leaves
phases at 0/π; rotation preserves magnitudes and moves every phase by θ;
translation preserves the *set* of magnitudes and permutes the sites
cyclically. Asserted directly on a test state (norm, phase, support).

**F2 (pass) — the orders and controls are as declared.** 6 unique orders
executed per mechanism, 3 single-piece arms each, 2 controls; the gain-only
arm reproduces BUILD-32's **16** exactly (setup comparability), and the
no-chaos baseline reproduces it too.

**F3 (pass) — the frozen metric per arm, and the invariants.** The
contamination total for every arm is reported; the first task is exact in
every arm; every arm's verdict fields equal the direct run's, field for field.

**F4 (pass) — the verdict, per the operator's rule and Rule 5.** Promotion
requires a **strict improvement** of the frozen metric with no invariant
regression; ties, worse, unstable or merely-different results are **rejected**
and preserved. The gate computes the verdict from the numbers and asserts the
report matches the rule.

**F5 (pass) — the rule-conflict record.** This document states which text was
applied (the hook's bounded discipline), that the build satisfies the notes'
exhaustive reading as well (3! = 6, all executed), and that no global
optimality claim is made.

## Out of scope, named (not half-built)

- **Wiring any piece into a stack**: nothing is promoted unless an arm passes
  F4 (the build-then-wire precedent).
- **Macro-position variants** (mid-stack, before the consumer readback):
  named, not merged.
- **The other stacks** (perception, closure): unchanged from BUILD-32's
  staging.

## Outcome (2026-09-27, after the code)

**THE VERDICT: CHAOS IS REJECTED AGAIN — 19 of 19 arms tied at the recorded
contamination 16, with zero strict improvements and zero worsenings.** 5/5
gates pass (F1–F6). The precommit's prediction — "at least one order produces
a measurable difference" — is **FALSIFIED** and stays on record, with a
better explanation measured in its place.

**F1 measured that the pieces are genuinely distinct operations** on the
carried state: gain scales every magnitude by the factor and leaves phases;
rotation preserves magnitudes exactly and moves every phase by θ; translation
preserves the magnitude multiset and permutes the sites cyclically (a
full-lattice translation is the identity). Nothing here is a renamed whole.

**F3/F4 measured all 19 arms** — no-chaos, gain-only, rotation-only,
translation-only, and the six orders of `{Gain, Rotation, Translation}`, each
under logistic r = 4 and under Hénon (a, b) = (1.4, 0.3) — every one at
contamination **16**, with the first task exact and every arm's verdict
fields equal to the direct run's, field for field (114 declared runs).

**F6 measured WHY, on the real system** (the actual dissipated residue from
task A plus task B's actual contribution):

- every piece really does change the carried state (asserted);
- **gain and rotation move bin powers by up to ~2.2e1 and ~1.4e1** — a real,
  phase-sensitive effect;
- **translation moves them by ~1.7e-10** — numerical noise, because a shift
  multiplies the residue's spectrum by a unit-modulus ramp and its cross term
  with the contribution is at the noise level here;
- and the consumer's **membership is unmoved in all three cases** (6 bins
  before and after).

So the consumer's readout is **affine-blind**: the perturbation is real and
the power spectrum moves, but the *set of bins that clear the absolute
threshold* does not — the bins that move a lot sit far above the threshold
(relative changes of ~1e-6), while the bins that sit near it (the residue's
own) move by less than their margin. BUILD-32's null result for the gain is
now generalised to the whole affine group, with numbers attached.

**TWO CORRECTIONS, both mine, both in the gates** (kept, never tuned):

1. F6's first version used a synthetic residue pattern whose flat-index
   structure made the translation *power-invariant by construction* — a
   fixture artifact, not a property of the system. Rewritten against the real
   carried residue (and the pad is not `Clone`, so the fixture is rebuilt
   deterministically per piece).
2. F5's assertions tripped twice on **line wrapping in my own document** (a
   quoted phrase split across lines is not `contains`-able). The fix was to
   rewrap the quoted text — not to loosen the check.

**Nothing was promoted, so nothing was wired.** The mechanisms remain a
candidate module (`chaos.rs`, 147 lines, now carrying the three affine pieces
and their declared parameter conventions); the two new pad seams (`rotate`,
`translate`) are declared end-of-stack state seams, inert when unused, and
BUILD-5's/BUILD-16's/BUILD-32's recorded gates re-pass unchanged.

**The rule conflict was applied, not resolved:** this build follows the
deployed hook's bounded wording AND satisfies the notes' exhaustive wording
(3! = 6, all executed), and F5 asserts that both texts are quoted in this
document. The notes-vs-hook discrepancy remains open for the operator.

**Suite:** 381 → **382 passed** / 1 failed (`hops_probe` by design, D61) / 5
ignored (live, opt-in) once the log line exists. L4 caught the missing log
entry a **NINTH** consecutive time.

**Line economy:** `chaos.rs` 113 → 147 (+34: the angle/offset conventions and
the `AffinePiece` enum); the gate file 427 lines; the engine gains two pad
seams.

## Files

- `crates/atom-cognitive-engine/src/chaos.rs` — `next_angle`, `next_offset`,
  `AffinePiece`.
- `crates/atom-cognitive-engine/src/scratch_pad.rs` — `rotate`, `translate`.
- `crates/atom-cognitive-engine/tests/build33_split_chaos.rs` (new) — F1–F6,
  the 19 arms, and the verdict.

## Rules

**Rule 1:** the pieces are intrinsic operations; nothing is promoted by
declaration. **Rule 2:** every unique order of the declared 3-piece set is
executed under matched controls; the claim is bounded to the executed orders
and no global-optimality claim is made. **Rule 3:** BUILD-32's result is
evidence, and the sibling project's promotions are not an oracle. **Rule 4:**
untouched — classical mechanisms, no quantum claim. **Rule 5:** two
mechanisms by intrinsic dynamics, matched no-chaos/single-chaos/multi arms,
frozen metric, and rejection unless the metric strictly improves. **Rules
6–7:** not engaged by this component experiment (no principal candidate is
claimed); the operator's non-retroactivity decision stands.
