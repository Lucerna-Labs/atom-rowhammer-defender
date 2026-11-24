# Ordering laws

Status: v0.1, 2026-08-21. Two enforcement layers:

1. **Machine-checked prerequisites** — stages declare prerequisites; the
   stack builder (`atom_cognitive-core::stages`) rejects cycles and
   unsatisfied prerequisites at assembly. Violating these is a *construction
   error*: the stack does not exist.
2. **Behavioral order laws** — orderings that are legal to assemble but
   demonstrably worse. Each has a permutation test proving a designated gate
   fails under the mis-order. Legal ≠ correct; the suite makes the
   difference measurable.

## Law table

| # | Law | Enforcement | Receipt |
|---|---|---|---|
| O1 | Time passes before learning (DECAY before BIND) | behavioral (permutation test: learn-before-forget fails revision/retention gates) | C.1 |
| O2 | Order/structure exists before ATTEND consumes it | machine-checked (structure-validation stage is a prerequisite of ATTEND) | C.2 |
| O3 | GATE before blend/route-merge; admission does not license an answer | behavioral + machine-checked (eligibility gate precedes assertion) | C.3 |
| O4 | Load (state, capabilities) before traffic (queries) | machine-checked (preload stage precedes query stages) | C.4 |
| O5 | Compositions are acyclic | machine-checked (builder rejects cycles) | C.6 |
| O6 | Writes are budgeted after the write (CONSERVE last on the write path) | behavioral (conserve-before-write fails capacity invariants) | C.1 |
| O7 | Never assert before retrieval+eligibility | machine-checked (assert stage requires retrieve+gate prerequisites) + behavioral | C.3 |
| O8 | Exploration decays monotonically; settle before assert | behavioral (no-cooling variant fails determinism/settle gates) | chaos-budget gates |
| O9 | Verify at boundaries; reordered state is rejected | machine-checked (hash verification stages at cycle ends) | C.7 |
| O10 | Weaken is harsher than strengthen | invariant check in REINFORCE/DECAY configs | C.8 |

## Divergence settled

`learn` is **attention-first**: ATTEND → (BIND | REINFORCE) → CONSERVE.
Evidence: the executable pipeline applies the attention gate before
nucleation (`repo:atom_cognitive_experiment.py:384` before `:393`), and
attention-first is what O2 requires (attend to what is there before creating
something new). The nucleation-first variant in
`repo:atom_runtime_knowledge.py:156` is recorded as superseded for this
project.

## Canonical cycle orders (normative)

- observe: `validate → predict → decay → attend → gate → write(bind|
  reinforce|revise) → conserve → verify`
- query: `validate → attend → gate → retrieve → assert_or_abstain → verify`
- consolidate: `validate → predict_cluster → bind_shared → decay_instances
  → conserve → verify`

Any deviation must either be rejected by the builder (prerequisite class) or
carry a permutation test showing which gate it breaks (behavioral class).
There is no third kind of deviation.
