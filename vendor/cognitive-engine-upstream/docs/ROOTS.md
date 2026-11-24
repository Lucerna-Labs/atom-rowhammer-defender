# The cognitive roots

Status: v0.3 revised 2026-09-25. Nine roots — PERCEIVE admitted at v0.3
(E1, docs/E1-PERCEPTION.md); EXPLORE demoted at v0.2 (see the demotion
log; decision D20). Each entry: definition, kind, invariants, evidence.
The admission filter (from the plan): a
candidate is a ROOT only if (1) **irreducible** — other candidates
decompose into compositions of roots, never the reverse; (2)
**load-bearing** — removing it breaks a measurable behavior; (3)
**composable** — participates in ordered, acyclic compositions; (4)
**evidenced** — grounded in receipts or cited literature. Candidates
failing the filter are rejected to compositions or demoted.

The roots are *cognitive* primitives. They are intentionally substrate-
independent: the seven universe primitives of atom-harness are one possible
implementation substrate, not an import.

## The root set

1. **ATTEND** — selection. Produce relevance weights over candidates from a
   cue; soft (weighting), never a hard decision.
   Evidence: attention recipe; predictive-processing precision weighting;
   receipt C.2 (order must exist before attention — attention operates on
   structure, it does not create it).

2. **BIND** — association. Form a durable link between a cue-pattern and a
   content-pattern where none existed (nucleation), within capacity.
   Evidence: bind/nucleate fork; pattern separation (A) is the match-gate
   failure mode of BIND.

3. **REINFORCE** — strengthening. Increase the weight of an existing link as a
   function of use and prediction success. Rate bounded; gentler than DECAY's
   rate (receipt C.8 asymmetry is a law, not a tuning accident: false
   positives must cost more to bury than to boost).

4. **DECAY** — weakening. Decrease the weight of links as a function of time,
   disuse, contradiction, and capacity pressure; retire at floors.
   Evidence: Ebbinghaus; ttl/support/mass decay laws; capacity retirement.

5. **PREDICT** — expectation. Given current state and a cue, emit the expected
   content *and* the error signal between expectation and observation
   (surprise is PREDICT's error output, not a separate root).
   Evidence: user-mandated; predictive processing; the homeostatic surprise
   observable; prediction-error learning (Rescorla–Wagner).

6. **REVISE** — contradiction resolution. When surprise/contradiction is
   detected against an existing link: **dissolve first, then replace**
   (forget-before-learn is REVISE's internal order law; receipt C.1).
   REVISE composes DECAY→BIND→REINFORCE but earns root status because its
   *trigger condition* (contradiction) and its *ordering law* are
   load-bearing independently — memory reconsolidation evidence; the
   revision gates in receipts.

7. **GATE** — routing/inhibition. A hard, context-dependent decision: pass,
   block, or redirect. Where ATTEND weights, GATE decides.
   Evidence: context-gating and guarded-relay stages exist because ungated
   transfer corrupted behavior (receipts C.3, C.9); prefrontal inhibition.

8. **CONSERVE** — budgets. Renormalize total state onto fixed budgets after
   writes; evict at capacity by a deterministic weakest-first rule.
   Evidence: conservation budgets; working-memory limits (A); conservation
   runs *after* the write (receipt C.1, embodied at
   `repo:atom_cognitive_experiment.py:396`).

9. **PERCEIVE** — hypothesis formation. Form a named, revisable
   hypothesis about a window of readings: a label plus the IDS of the
   readings that supported it. Never a score, never a threat assessment,
   never a decision (judgment is a separate class — the sensing /
   perception / judgment class law, receipted 2026-09-24). Kind: a
   cognitive primitive over EXTERNAL readings; the other eight roots
   operate on internal memory state, which is the class-grounded
   irreducibility argument. Invariants: evidence references, not fused
   numbers (anti-collapse law — no confidence field exists in the
   mechanism); revisable — a contradicting window supersedes, and the
   retracted entry is retained, never silently overwritten; sits on top
   of the readings so a consumer can always fall back to them; abstains
   below the integer support floor and on tied evidence. Evidence: the
   pinned definition receipted 2026-09-24 ("the V case is nameable only
   JOINTLY"); E1 admission gates (docs/E1-PERCEPTION.md) — the ablation
   `omit(perceive.perceive)` drops naming 9/9 → 0/9 (load-bearing); the
   best single-dimension naming rule on the E1 corpus is 6/9 (joint-only);
   supersession chains retain history (revisability); the type is
   structurally score-free (anti-collapse). Admitted v0.3 (E1,
   2026-09-25). Re-examination clause: if a composition of the other
   roots ever reproduces revisable, evidence-referenced naming over
   readings, this admission must be re-examined — the same discipline
   that demoted EXPLORE.

## Demotion log

- **EXPLORE** — admitted v0.1, **demoted at v0.2 (decision D20)**. The
  admission filter requires a failing ablation; v1's exploration mechanisms
  (seeded tie-breaking among exact similarity ties, monotonic formation
  cooling) are behaviorally inert — no current program distinguishes them
  from order-determinism. The candidate strengthening (annealed match
  acceptance below threshold during formation) is *contra-indicated by the
  receipts*: gate-before-blend and routed-memories-beat-blended-ones both
  say imprecision in matching corrupts rather than explores. The ecosystem's
  one exploration-positive result (homeostatic uphill acceptance during law
  formation) lives in policy search, not trace memory. EXPLORE may re-earn
  root status if this engine grows law-formation machinery with something
  to explore. The cooling and tie-break code remain as engine mechanisms,
  invariant-tested, claiming no root status.

## Composition table (candidates → root compositions)

| Candidate operation | Composition |
|---|---|
| remember | DECAY → BIND → REINFORCE ("apply time, then reshape") |
| learn | ATTEND → (BIND \| REINFORCE) → CONSERVE — **attention-first** (divergence settled, see DECISIONS) |
| retrieve | ATTEND → GATE → mixture → assert-or-abstain |
| forget | DECAY (+ CONSERVE eviction) |
| perceive | ATTEND → PERCEIVE → GATE (assert-or-abstain; the perception cycle, E1) |
| abstract | repetition/consolidation trigger → BIND shared structure → DECAY instances |
| consolidate | offline pass: PREDICT over stored content → low-surprise clusters → abstract |
| curiosity | formation-regime variation driven by PREDICT's error entropy (mechanism, not a root, per D20) |
| interference | not an operation — the failure mode GATE exists to govern |
| homeostasis | PREDICT + CONSERVE at the system boundary (governor lane, out of v1 scope) |

## The canonical cycles (see ORDERING.md for the laws)

- **Observe cycle** (write path): validate → PREDICT (expectation + surprise)
  → DECAY (time passes first) → ATTEND → GATE (bind-or-nucleate fork) →
  BIND/REINFORCE/REVISE (the write) → CONSERVE (budget last) → verify.
- **Query cycle** (read path): validate → ATTEND → GATE (eligibility) →
  mixture → assert-or-abstain → verify. Never assert before GATE.
- **Consolidation cycle** (offline): PREDICT over stored state → cluster by
  low surprise → BIND shared structure → DECAY instances → CONSERVE → verify.

## Ablation obligations (what makes each root load-bearing here)

Each root must have at least one engine gate that fails when the root is
disabled. The permutation suite in `atom-cognitive-engine` implements these
oblations as tests. A root with no failing ablation is demoted at the next
revision of this document.
