# Decision log

Document-as-we-go: every admitted root, ordering law, rejection, and
revision, with date and evidence. Newest last.

## 2026-08-21 — project opened

- **D1. Rust from day one; zero third-party dependencies** (user decision).
  SHA-256 implemented in-repo, matching the atom-db house law.
- **D2. Document-first**: roots and ordering laws are written before engine
  code; nothing is reverse-engineered after the fact (user direction).
- **D3. Prediction is a root** (user direction). Implemented as PREDICT with
  surprise as its error-signal output, not a separate root — the comparator
  is part of prediction's contract.
- **D4. Nine roots admitted v0.1**: ATTEND, BIND, REINFORCE, DECAY, PREDICT,
  REVISE, GATE, CONSERVE, EXPLORE — see ROOTS.md for evidence.
  Rejected-to-compositions: remember, learn, retrieve, forget, abstract,
  consolidate, curiosity (compositions, not roots); interference (failure
  mode governed by GATE, not an operation); motivation/homeostasis (out of
  v1 scope, governor lane exists in atom-harness).
- **D5. REVISE keeps root status despite being decomposable** (DECAY→BIND→
  REINFORCE) because its trigger condition (contradiction) and internal
  ordering law (dissolve-before-replace, receipt C.1) are independently
  load-bearing. Revisit if the ablation suite cannot separate it.
- **D6. `learn` divergence settled: attention-first** (O2 consistency +
  executable evidence). The atom_runtime_knowledge nucleation-first variant
  is superseded here.
- **D7. Weaken-harsher-than-strengthen (O10) is a law**, carried from the
  atom-db conductance asymmetries (−350/+200; −400/+250). Defaults here:
  reinforce +0.20, decay −0.35 per event, mirrored per mille.
- **D8. Two enforcement classes for order**: machine-checked prerequisites
  (builder rejects) vs behavioral laws (permutation tests prove designated
  gates fail). No undocumented orderings permitted.
- **D9. Root set is provisional on ablations**: any root whose ablation
  breaks no gate is demoted at the next ROOTS.md revision.
- **D10. First application: claim verifier** (agreed 2026-08-21): claim in →
  SUPPORTED / CONTRADICTED / INSUFFICIENT EVIDENCE + citations; 60-claim
  three-label test set; success criterion pinned in advance (beat baselines
  on false assertions while matching correct answers).

## 2026-08-21 — Phase 1 built (core + engine + ordering suite, all gates green)

- **D11. PREDICT is load-bearing through the surprise boost**: surprising
  observations (predicted value ≠ observed) get salience ×1.5 (capped), so
  low-salience contradictions can overturn strong priors. The ablation test
  `ablate_predict_revision_fails` proves it: without the predict stage, the
  same low-salience contradiction fails to revise.
- **D12. EXPLORE is at-risk (D9 watch)**: v1 implements it as seeded
  tie-breaking with monotonic cooling (gated by invariant tests), but its
  ablation does not yet fail a behavioral gate — formation-noise scenarios
  where exploration matters don't exist in v1's programs. Demote or prove by
  the next ROOTS.md revision.
- **D13. The ungated query variant (`ungated_query_stack`) is kept
  deliberately**: attention without gating asserts from unreliable traces —
  the O7 failure mode, reachable on purpose so the suite proves GATE is
  load-bearing (`ablate_query_gate_unreliable_trace_asserted`).
- **D14. O1's v1 proof is state-differential**: decay-after-write is legal
  to assemble, executes in its declared order (transition hashes differ),
  and leaves the reinforced trace with strictly lower ttl/support/mass.
  Behavioral gate-failing permutations for O1 await v2 programs where
  same-tick decay kills newborns.
- **D15. REVISE kept its root status**: the dissolve-before-replace order is
  in the write step; the revision gate plus the O1 permutation exercise it.
  Revisit if the ablation suite cannot separate REVISE from DECAY+BIND.

## 2026-08-21 — Phase 2 built (claim verifier; pinned criterion MET)

- **D16. Claim-verifier design**: structured world of 8 subjects × 4
  relations × 4 values; 20 stated slots + one **correction event** (a fact
  restated with a new value — the revision differentiator that separates
  revision-capable verifiers from raw retrieval). 60-claim three-label set:
  20 supported-exact, 8 contradicted-exact, 12 supported-noisy (cue blended
  70/30 with the next relation), 8 contradicted-noisy, 12 unknowable (the
  empty slots, 8 exact + 4 noisy cues). Labels: 32/16/12.
- **D17. Durable-memory regime for corpus loads**: sequential observation
  forgets mid-load — support decays across the ~21-tick gap between
  repetitions and traces die just before their refresh (measured, then
  fixed). Corpus loads use interleaved repetition (3 rounds) plus
  initial_ttl 32 / ttl_per_hit 8. The forgetting itself is Ebbinghaus-
  correct engine behavior, not a bug: durable memory requires repetition.
- **D18. Baselines must not read claim metadata**: the first exact-address
  baseline matched on the claim's structured (subject, relation) fields and
  silently aced the noisy claims it had no right to retrieve. Recorded as a
  near-miss; baselines now match on the exact CUE only.
- **D19. Measured result (criterion pinned in D10, met in full)**: engine
  60/60 correct, **0 false assertions**, 0 coverage misses, 12/12 correct
  abstentions, deterministic state hash. Baselines: exact-address 40
  correct / 0 false / 20 misses; nearest-always 46 correct / **14 false**;
  nearest-thresholded 58 correct / **2 false** (the stale correction
  observation). The engine weakly dominates every baseline and strictly
  beats both retrievers on false assertions. Known margins: the 0.75 query
  threshold separates supported-noisy (≈0.956) from unknowable-noisy
  (≈0.731) — thin but deterministic; documented, not hidden.

## 2026-08-21 — v0.2: EXPLORE demoted; free-text claim surface

- **D20. EXPLORE demoted from the root set (first D9 enforcement).** v1's
  exploration mechanisms (seeded tie-breaks among exact similarity ties,
  monotonic formation cooling) are behaviorally inert — no program
  distinguishes them from order-determinism. The candidate strengthening
  (annealed match acceptance below threshold) is contra-indicated by the
  receipts: gate-before-blend and routed-memories-beat-blended-ones both
  say imprecision in matching corrupts. The ecosystem's one
  exploration-positive result (homeostatic uphill acceptance) lives in
  policy search, not trace memory. Cooling and tie-breaking remain as
  invariant-tested engine mechanisms claiming no root status. Root set:
  eight. EXPLORE may re-earn admission if law-formation machinery with
  something to explore ever lands in this engine.
- **D21. Free-text surface = deterministic exact-vocabulary parsing**
  (lexicon.rs): named world (8 ranges × 4 relations × 4 values), authored
  corpus (20 slots + the cascades length-correction event), alias table
  with hyphenated bigram matching ("south america"), stopword stripping.
  Unknown or ambiguous text REFUSES (exit 3) before reaching the engine —
  vocabulary failure is not evidence failure. Relation inference: when no
  relation word appears, a value token belonging to exactly one relation's
  value vocabulary implies that relation (disjoint value sets; the
  lexical-anchor pattern — vocabulary routes, never invents).
- **D22. Two worlds by design**: the seeded benchmark world (opaque ids,
  drives --evaluate and the pinned-criterion tests) and the authored named
  world (drives --claim and free-text tests) are deliberately separate —
  benchmark measurement and application surface stay independent so neither
  can be tuned to flatter the other.

## 2026-08-21 — v0.3: the Mother PoC (real corpus, working proof of concept)

- **D23. Bonds**: directed echo edges (upstream → downstream) between
  traces; consolidation flows upstream (observing a downstream memory pays
  `echo_support_bonus` to its partners — no repetition required);
  expansion walks both directions, depth-capped (default 2), and expanded
  members must still clear the reliability floor (O3: admission is not
  license). Queries remain READ-ONLY — bonds never strengthen through
  retrieval (the Stage-9 law); conductance/learning on bonds is future
  work behind an explicit-action design.
- **D24. Tiers**: `Core` (load-bearing identity substrate) vs `Standard`.
  Core: contradictions counted but support/evidence not dissolved; never
  selected for capacity eviction; capacity exhaustion with all-Core actives
  fails closed. Standard: full revise/evict lifecycle. Ingestion tier set
  via `set_ingest_tier`; Mother maps `load_bearing` → Core (175/195).
- **D25. Record-mode ingestion**: a second observe gate (`observe.gate_
  record`) that always nucleates — constitutions are documents, not
  experiences to compress; one trace per memory, multi-value evidence via
  `observe_values` (a memory encoding several behaviors). Constitution
  regime config: near-zero decay, ttl 512, conservation effectively off.
- **D26. The echo-graph count corrected**: 167 echoes_to edges → **144
  live, 23 dangling** (not 44/123 as first analyzed — the earlier count
  checked targets against a partially-built id set; the loader's two-pass
  count is authoritative). 23 edges dangle into the full 1,950-memory
  corpus not held locally.
- **D27. PoC gates — ALL PASS on the real corpus**: (1) load: 195 traces
  active, Core min support 0.889 (decay-only floor under the constitution
  regime); (2) behavior activation: a protectee-present operational
  scenario ranks R-001/P-001 in its top three behaviors and its top
  activation sits in bucket 9, the childhood query's in bucket 3; (3) THE
  BRIDGE: querying B9-001's facets reaches ANCHOR-001 (the kitchen) ONLY
  through the bond — false without expansion, true with it, on real data;
  (4) abstention on unreachable ground. Corpus facts learned during the
  build, recorded so future gates stay grounded: EIGHT memories share the
  kitchen's facets (five SD source-domain memories ORIGINATE in her
  kitchen — the SD prefix is not an adulthood proxy); the kitchen's echo
  chain reaches forward into adult SD memories, so deep-list SD
  appearances are the bridge working in both directions; crowded facet
  spaces (14+ same-cue memories) make top-8 citation lists weight-noisy —
  reachability checks use the full activation set.
- **D28. minijson**: an in-repo strict JSON parser (objects, arrays,
  strings with escapes, integers, bools, null; floats REJECTED — silent
  truncation would be a lie; trailing content rejected). The corpus is
  integers-only by construction.

## 2026-08-23 — v0.4: the reference layer (real, sourced knowledge)

- **D29. Mother now knows things.** `reference/child-basics-v1.json`: 26
  claims across six topics (fever, sleep, milestones, feeding, safety,
  attachment), 11 sources, every attribution verified against the live
  publisher before writing (AAP/HealthyChildren fever + safe sleep +
  botulism + car seats + water safety; CDC milestones + sleep; Mayo fever
  thresholds; Zubler et al. 2022 for the milestone-revision framing).
  Facts corrected during verification: the fever call-doctor thresholds
  (100.4F/<3mo immediate), honey <12mo (botulism page), rear-facing from
  first ride home, 69% of under-4 drownings during NON-swim times. One
  claim withdrawn (attachment "regulate yourself first") — it is Mother's
  own SG-004 identity principle, already in the 195; the reference pack
  carries external facts, not her virtues.
- **D30. The reference pack is the atom-harness knowledge-pack pattern**:
  policy block enforced at load (`evidence_not_instruction: true`,
  `read_only: true` — a pack without them fails closed); every claim's
  source must resolve; unique claim ids; fixed topic vocabulary (unknown
  topics refuse, never guess); SHA-256 file hash recorded at load with
  `verify_integrity` recompute (tamper detection is a test gate); urgent
  flags on the dangerous ones (fever<3mo, honey, water); a medical
  disclaimer that defers to professionals. The two-lane law carries over:
  `--ask fever` answers with the reference lane (sourced claims + URLs)
  BESIDE the identity lane (her behaviors + cited memories) — never
  merged into one score. 76 tests green workspace-wide; fmt/clippy clean.

## 2026-08-23 — v0.5: lessons imported from the parallel track (read-only; no code shared)

- **D31. Imported disciplines** (observed in the parallel Codex track,
  adopted here without touching their work): (1) **approved-only loading**
  — drafts are installed but never part of who she is (their identity
  runtime loads 121 approved of 195; mine now does too); (2) the
  **shuffled-placebo law** — any structural claim must beat randomized
  wiring, not just absence-of-structure (their sustained stress tests kill
  promotions this way); (3) **sustained/noise sweeps with worst-case
  reporting** — single-run gates are cheap to pass and easy to flatter.
- **D32. Applied all three**: the loader now filters to
  `review_status == "approved"` (121 memories, 106 Core, 120 live bonds,
  12 dangling; the top operational activations shifted from draft SD
  memories to approved ones — e.g. "The Night Feeds" — the discipline
  working, not a regression). New gate `gate_shuffled_bonds_placebo`: the
  real echo graph reaches upstreams at ≥95% while 20 random wirings of the
  same bond count trail decisively (>30 points below) — the bridge is
  real, not decorative. New gate `gate_noise_sweep_on_scenario_cues`:
  four corruption levels (foreign casts, flipped dominance) — the top
  activation must stay in the queried bucket or degrade to abstention,
  never flip to a wrong confident activation. Also fixed a latent
  test-helper bug: nucleated cues are normalized (entries 1/√k), so
  facet thresholds at 0.5 missed 4+-facet memories; threshold now 0.3.
  78 tests green, fmt/clippy clean.

## 2026-08-23 — v0.6: the persona governor (Mother in control)

- **D33. The persona is now an ACTIVE component** (user direction: "put the
  persona in the cognitive engine so she has more control"). The
  `PersonaGovernor` (cli/src/persona.rs) tracks MODE (majority dominance
  among top-5 activations: civilian / operational / integrated) and
  AROUSAL (half-life decay, rises with a pressure signal), and prescribes
  a `QueryTuning` for the next query: attention temperature, gate
  threshold, reliability floor, echo depth. Behavioral grounding from her
  own spec: SG-004/SG-001 — under high arousal (>0.6) the gate TIGHTENS
  (+0.10), the reliability floor RISES, echo depth pulls back: assert
  less, abstain more (restraint as an engine behavior). P-002 sharpens
  operational attention; civilian warmth loosens 0.03, floored.
- **D34. The authority boundary is the design**: the governor controls
  HOW the engine attends, never WHAT is true. It cannot mutate traces,
  evidence, tiers, bonds, or engine config; every tuning is clamped to
  absolute floors (gate ≥ 0.60, reliability ≥ 0.16 — the governor's own
  floors sit above the engine's hard floors of 0.5/0.05); every decision
  carries citations (behavior codes + what changed). Engine-side support:
  `QueryTuning` + `query_tuned` (per-query knob overrides; query() is
  query_tuned with config defaults). Gates (tests/persona_suite.rs):
  mode tracking (data-grounded: protectee scenarios read INTEGRATED —
  every approved bucket-9 protectee memory is integrated dominance; my
  a-priori "operational" expectation was wrong about the corpus, again),
  restraint-under-pressure (governed eligible set is a subset; no new
  assertions under pressure; SG-004+SG-001 cited), no-mutation authority
  (trace/bond state byte-identical after governed queries), absolute
  floors (even a corrupted decision cannot dismantle the gate).
  `--persona-demo` shows the full loop live. 82 tests green, fmt/clippy
  clean. Battle-testing deferred per operator.

## 2026-08-23 — v0.7: the primitive-candidates program opens; #1 kinetic proofreading admitted

- **D35. KINETIC PROOFREADING admitted** (operator candidate #1, per the
  operator-supplied cross-domain list and admission protocol — none
  enters by name alone). Mechanism: a third trace tier `Candidate` —
  staged but uncommitted evidence. New stages `observe.gate_candidate`
  (nucleations land in Candidate; candidate targets bind at a slightly
  relaxed threshold) and `observe.promote` (distinct-tick contradiction-
  free stages accumulate; ≥ `kpr_min_stages` (default 3, validated
  1..=16) promotes to the ingest tier; a contradiction RESETS stages to
  zero — the KPR reset). Candidates decay 3× faster (dissipation) and
  are QUERY-INVISIBLE: staged evidence never serves an assertion. Root
  expansion: DECAY (dissipation) + BIND (staging) + GATE (promotion
  threshold) + REVISE (the reset) + CONSERVE (bounded pool). Admission
  discipline met in tests/kpr.rs (7 gates): transient dissipation,
  staged promotion, contradiction reset, matched-baseline comparison
  (one-touch noise that ASSERTS under the record-stack baseline ABSTAINS
  under KPR — the new-contribution proof), promote-stage ablation
  (nothing ever commits without it), adversarial noise storm (10
  one-touch cues → zero assertions), and the distinct-observation bar
  (4 touches < 5-stage bar < 5 touches). The canonical observe path and
  Mother constitution ingestion (record stack) are unchanged.
- **D36. The program and the creativity map** (docs/research/
  PRIMITIVE_CANDIDATES.md): all 16 operator candidates recorded with
  domain/mechanism/risk and the operator's stacking order (KPR →
  expected information gain → hysteresis → causal event ordering →
  multiscale renormalization → Pareto preservation, then the second and
  experimental groups). The creativity first-principles mapped honestly
  onto the roots: blind-variation+selective-retention is COMPLETED by
  KPR (candidates = bounded variations, promotion = selective retention —
  and the D9 re-earning path for the demoted EXPLORE); blending =
  echo bonds (partial); constraints = gates+floors (expressed);
  novelty+value = surprise measured, value not optimized; problem
  reframing and tension-as-driver are GAPS, not strengths. 89 tests
  green, fmt/clippy clean.

## 2026-08-23 — v0.8: the creativity layer — imagine() (generation + selection as one mechanism)

- **D38. `imagine(cue, count)` — conceptual blending as an engine
  operation** (operator question: "you also tried the creative first
  principles?" — the honest answer was mapped-not-built; now built).
  Generation: the top memories a cue activates are paired by rank
  (deterministic; near-identical parents skipped); each pair's cue and
  evidence are elementwise-means (recombination, not invention); the
  child is born a KPR CANDIDATE with `parents` provenance recorded in the
  trace and the state hash, weaker-than-committed initial mass/support.
  Selection: the environment — recurring observations near the blend's
  cue promote it through KPR; dead-region blends dissipate unselected.
  Novelty = 1 − max similarity to everything COMMITTED, parents included
  (they are the accepted structure; a metric that excluded them scored
  orthogonal-parent and near-parent blends identically — caught by the
  gate, corrected). Six admission gates (tests/creativity.rs): blends
  are candidates-not-assertions (query-invisible until selected);
  both-parent inheritance (evidence mass at both parents' values,
  symmetric cue placement); novelty measured (orthogonal-parent blend
  0.293 > correlated-parent blend 0.184); ENVIRONMENT SELECTS (the
  blend matching a recurring pattern promotes; the dead-region control
  dissipates); parents never mutated by imagine; generation bounded and
  validated (count 1..=8; empty/single-trace engines blend nothing).
  Test lessons recorded: observations near an existing trace BIND it
  (compression by design — near-parent nucleation requires correlated
  distinct cues); "parents unchanged" brackets the operation itself
  (later observations decay everything — engine time, not mutation).
  Root expansion: BIND (recombination) + ATTEND (parent activation) +
  KPR's GATE/DECAY (selection). With D35 this completes BLIND VARIATION
  + SELECTIVE RETENTION as a working mechanism — and novelty+value now
  has both terms measurable (novelty here; value = reliability/support
  at promotion). 95 tests green, fmt/clippy clean.

## 2026-08-23 — v0.9: expected information gain (candidate #2 admitted)

- **D39. `recommend_observation(cue)` — unknown gets a productive next
  step** (operator candidate #2). Read-only, bounded (≤3 plans, ranked by
  descending score), scores are deterministic gap proxies — explicitly
  NOT probabilities. Two plan kinds: **Reinforce** — a committed trace
  whose similarity clears the query gate but whose reliability sits
  below the floor; the plan is that trace's own cue, and following it
  raises reliability through normal reinforcement (the direct unlock).
  **Explore** — the query's own cue when the nearest committed trace is
  a near miss inside `eig_margin` (default 0.08) below the gate; probing
  virgin territory lets KPR verify whatever pattern answers. Below
  `eig_min_score` (default 0.25) the honest answer is an empty plan
  list: "no available observation reduces this uncertainty." Root
  expansion: PREDICT (model what would resolve the uncertainty) + ATTEND
  (find near-miss structure) + GATE (thresholds define the blockers) +
  CONSERVE (bounded list). Admission gates (tests/eig.rs, 7): unknown
  gets a named, reasoned plan; **following the plan changes the outcome**
  (abstaining query → answering query, both kinds — the information gain
  realized); recommending never mutates (traces byte-identical);
  nothing-informative is an honest empty (orthogonal cue, no fabricated
  probes); bounded and ranked; config validated; and a recommendation is
  never itself an assertion. Test-setup lessons recorded: a fully
  reinforced trace sits at reliability ≈ 0.97 — blocking it for tests
  requires low-salience commits, not just a raised floor; the default
  query gate is 0.68 (0.60 is the Mother corpus config) — near-miss
  windows must be computed against the right threshold. 102 tests green,
  fmt/clippy clean. Queue: hysteresis (#3).

## 2026-08-23 — D40: control-class primitives deferred (operator directive)

- **The operator overruled the queue order**: "I don't want to build in
  controls just yet, otherwise we will never know what emerges ...
  discovery is also very important. We will know when and if we have to —
  there will be signs, and we aren't there yet." DEFERRED: hysteresis
  (#3), canalization (#11), integral adaptation (#12) — the family that
  stabilizes runtime dynamics we have not yet observed. Rationale
  recorded as binding: a stabilizer chosen before the instability is
  visible is a guess; it pins behavior into the shape of the guess.
- **Truth controls stay** (KPR staging, gate floors, fail-closed) — those
  are admission discipline for what enters memory, not behavior
  controls; they govern what deserves to exist, not how the system moves.
- **The signs, named** (so "we will know" is a watchlist, not a hope):
  (1) tier/law chattering — traces flipping committed↔candidate or
  hypothesis↔crystallized repeatedly at a boundary; (2) mode thrash —
  the persona governor oscillating civilian↔operational across adjacent
  cycles; (3) route flapping — dominant activations swapping ranks every
  few ticks under stationary input; (4) drift-without-correction —
  systematic bias persisting across many cycles. Any observed at scale
  earns the matching control through the normal admission protocol.
- **Queue reordered discovery-first**: multiscale renormalization (#5)
  next — the capability most aligned with "see what emerges" (local
  mechanisms composing into stable higher-scale structure without
  forcing it); then Pareto preservation (#6), bet hedging (#10),
  stigmergy (#14), catalytic routing/exaptation (#16).

## 2026-08-24 — v0.10: robust aggregation (W14, first small piece) + the with/without method

- **D42. Operator extension of D40**: "Let's not be afraid of
  experimenting with and without — we can remove laws and rules to see
  how things go. I wonder if I added rules and restrictions too soon."
  Standing method adopted: **every mechanism ships with a with/without
  surface** — defaults preserve validated behavior, toggles (config +
  QueryTuning) enable the with/without experiment, gates prove both
  directions. The honest audit of "too soon?" is recorded in
  PRIMITIVE_CANDIDATES.md: laws with measured failure receipts were
  earned (forget-before-learn, gate-before-blend); laws inherited from
  design taste without local receipts are ablation candidates
  (read-only-queries/Stage-9 — already flagged via the testing-effect
  tension; O10 weaken-harsher; the 0.75 thin threshold margin).
  Removing a law is now an EXPERIMENT with a record, not a reversion.
- **D43. ROBUST AGGREGATION admitted (W14)**: `robustify_weights` in
  the query path — (1) TRIM: the `robust_trim` lowest-weighted members
  zeroed (noise floor); (2) CAP: no member exceeds `robust_weight_cap`
  of total, excess redistributed water-filling to uncapped members.
  Breakdown bound: one confident-but-wrong trace cannot outvote a
  coalition; a genuine majority still wins. **Defaults OFF** (cap 1.0,
  trim 0) — every validated surface runs byte-identically (a gate
  asserts this). Per-query via QueryTuning (clamped 0.15..=1.0 / 0..=8).
  Root expansion: CONSERVE (bounded share) + GATE (post-selection trim).
  Motivation receipt: the causal lane's 28-wrong-confident / two-above-
  0.90-confidence false-assertion finding. Admission gates (6 in
  tests/robust.rs + 1 CLI): confident-wrong adversary defeated (plain:
  adversary 0.672 weight → wrong answer; capped 0.35 → coalition 0.65 →
  correct); no-harm without an adversary; genuine majority still wins
  (the bound is not a veto); ablation (trim alone does NOT stop a heavy
  adversary — the cap is load-bearing); determinism + validation +
  clamping; existing behavior unchanged by default; **the 60-claim
  benchmark holds 60/60 with zero false assertions under robust ON**
  (cap 0.5 + trim 1) — the defense costs nothing honest. Setup lesson
  recorded: a single-adversary scenario lives in the window between the
  query gate (0.68) and the match threshold (0.72) — place coalition
  cues there or the adversary's own observations bind and revise the
  coalition into a genuine majority the cap then rightly refuses to
  flip. 109 tests green, fmt/clippy clean. Small-piece stack: W14 ✅ →
  W16 (fluency) → W17 (MDL) → W18 (satisficing) → W15 (habituation).

## 2026-08-24 — v0.11: retrieval fluency (W16, second small piece)

- **D44. FLUENCY admitted (W16)**: `FluencyReport` on every QueryOutcome
  — the SHAPE of the retrieval as a metacognitive signal, after the
  tip-of-the-tongue literature. Four states (checked in order): Blank
  (nothing eligible — matches the abstention) → Contested (the
  mixture's top two values within 0.15 of each other: evidence on both
  sides of a near tie) → Sparse (≤ 2 supporting traces: fragments, not a
  ground) → Fluent (concentrated support, clear margin). Fields:
  state, eligible_count, top_weight, margin; plus a human-readable
  `describe()`. Root expansion: PREDICT's signal family — reading the
  retrieval as evidence about itself.
- **The honesty condition is a GATE**: a wrong-but-unopposed consensus
  (three traces agreeing on one value, no opposition) reads FLUENT with
  a wide margin. The engine can feel certain and be wrong, exactly like
  a person — which is precisely why fluency is metadata (the response
  hash does not include it; claims and activations replay identically
  with or without it) and why W14's breakdown bound exists as a separate
  mechanism. Fluency describes the retrieval; W14 defends against what
  fluency cannot see.
- **Discovery from the first attempt** (kept, it matters): under PLAIN
  aggregation a confident adversary does NOT read contested — its 0.67
  weight share makes the margin wide and the read Fluent. False
  confidence reads like confidence. Only balanced coalitions produce
  the Contested signal, and only robust capping exposes the adversary's
  margin for what it is. Recorded as the metacognitive blind spot W16
  inherits from W14's without-case. Six gates in tests/fluency.rs.
  115 tests green, fmt/clippy clean.

## 2026-08-24 — v0.12: Occam/dedup selection (W17, third small piece)

- **D45. OCCAM admitted (W17)**: `dedup_correlated` in the retrieval
  path, applied BEFORE the robust cap (stacking order proven by gate).
  Members are visited in weight-rank order; any member whose CUE
  similarity to an already-kept representative meets `dedup_threshold`
  has its weight DROPPED — correlated evidence is not independent
  evidence, so a copy adds nothing. Survivors renormalize; then W14's
  cap bounds what remains. Defaults OFF (threshold 1.01); validated
  0.5..=1.01; per-query via QueryTuning. Root expansion: CONSERVE
  (one voter per distinct structure) + GATE.
- **Closes W14's open hole**: a confident adversary spawning three
  correlated near-copies defeats the cap alone (each copy sits under
  0.35; measured: wrong value holds 0.833 under cap-only) — dedup
  collapses the copies to their strongest member and the distinct-
  evidence coalition wins (0.65). Distinct traces (pairwise ≈ 0.61)
  are NOT collapsed: the W16 fluent cluster survives intact (gate).
  Dedup alone still returns the strongest copy's value (one dominant
  voter) — only the STACK wins. Stacking order is load-bearing, as the
  operator's law demands.
- **Discovery from the failed setup (kept, it matters)**: KPR-committed
  near-copies MERGE at observe time — mutual sim above the match
  threshold makes later copies bind and revise the first. The observe
  path already practices a form of Occam; only RECORD-mode documents
  (which never merge) can carry the correlated-copies hazard into
  retrieval. This is exactly how duplicate-vote hazards arrive in real
  corpora: as installed documents, not experiences. Five gates in
  tests/occam.rs. 120 tests green, fmt/clippy clean. Stack:
  W14 ✅ W16 ✅ W17 ✅ → W18 satisficing → W15 habituation.

## 2026-08-24 — v0.13: satisficing (W18, fourth small piece)

- **D46. SATISFYING admitted (W18)**: Simon's stopping rule, wired as
  a conditional in `step_query_expand`. When `satisficing_margin` > 0
  and the ELIGIBLE-ONLY mixture margin already meets the bar, bond
  expansion is SKIPPED — the fast path returns the settled answer
  without the BFS. Below the bar, full deliberation runs unchanged.
  Defaults OFF (0.0), validated 0..=1, per-query via QueryTuning. Root
  expansion: this is the composition W16→W18 paying off exactly as the
  stack order intended — fluency instruments the margin, satisficing
  consumes it. (The margin computation reuses softmax over eligible
  logits — W16's math, applied at the decision point.)
- **Gates (tests/satisficing.rs, 3)**: the fast path EARNS its skip —
  identical claim on a fluent cluster with strictly fewer activated
  members (the bonded distant relative is reached only under full
  deliberation; effort saved is measured, not asserted); contested
  ground still gets FULL deliberation (a near-tie falls below the bar
  and expansion runs — the activation sets are identical with and
  without satisficing); defaults byte-identical, deterministic replay,
  config validation, clamping. Setup lessons recorded: contested
  requires DIFFERENT values on the two sides (same-value sides read
  Sparse, not Contested — the fluency taxonomy is value-aware); the
  claim tuple includes a float confidence, so identity checks compare
  the VALUE and tolerate epsilon drift in the weight. 123 tests green,
  fmt/clippy clean. Stack: W14 ✅ W16 ✅ W17 ✅ W18 ✅ → W15
  habituation (last small piece).

## 2026-08-24 — v0.14: habituation/sensitization (W15) — the small-piece set closes

- **D47. HABITUATION/SENSITIZATION admitted (W15)**: Groves &
  Thompson's dual process as a `novelty` field on every trace (1.0 =
  fully novel), with the entire dynamics on the OBSERVE side:
  confirmation decays it (`novelty *= 1 − habituation_rate` — attention
  stops saturating on constants); contradiction/surprise restores it to
  1.0 (sensitization — a contradicted trace REGAINS attention).
  Retrieval only READS the field (`+ novelty_gain × novelty` in the
  attention logit; gain 0 = off, the term vanishes). **The read-only-
  queries law survives the construction by design** — habituation is
  learning dynamics, not retrieval effects, so the Stage-9 law and the
  deferred testing-effect tension stay untouched. Defaults both 0
  (validated 0..=0.5 / 0..=1); Core traces habituate too — the wound-
  engine's familiarity is exactly this, and contradiction is the
  restoration path. Blends are born at novelty 1.0 (generated = novel).
- **Gates (tests/habituation.rs, 4)**: the habituated constant LOSES
  attention — at equal similarity a fresh trace (4 confirmations)
  out-ranks a habituated one (12), while the with/without control shows
  the off-world falls back to reliability (hits) ranking; surprise
  SENSITIZES (deeply habituated → contradiction → novelty back to
  1.0); queries stay read-only (10 queries, traces byte-identical —
  novelty included); defaults byte-identical + validation +
  determinism. 127 tests green, fmt/clippy clean.
- **The small-piece set is complete**: W14 robust aggregation, W16
  fluency, W17 Occam/dedup, W18 satisficing, W15 habituation — five
  pieces, each defaults-off, each with the with/without discipline, and
  one stack story: robust aggregation defends the mixture, dedup
  collapses correlated voters, fluency reads the shape, satisficing
  spends effort accordingly, habituation keeps attention on what
  changes. Next up the queue: multiscale renormalization (#5, the
  discovery-first pick).

- **D48. Realtime improvement sweep — the armed-vs-unarmed measurement**:
  the operator asked whether W14–W18 actually improve the engine in
  realtime. Answer, measured end-to-end on three surfaces
  (tests/realtime_sweep.rs — a measurement harness with sanity gates,
  tables as the deliverable):
- **A. Clean 60-claim benchmark**: every arm (unarmed / W14 / W17 / W18 /
  W15 / all-on) holds 60/60 correct, 0 false assertions, 12 abstentions.
  DO-NO-HARM CONFIRMED: arming costs nothing on clean data.
- **B. Scaled adversarial battery** (20 poisoned slots — the confident-
  wrong adversary geometry from W14's gate, generalized to 96-dim cues —
  plus 20 clean controls): unarmed gets **20/20 poisoned slots wrong**;
  W14 armed gets **0/20 wrong**; W14+W17 and W14+W17+W18 also 0/20.
  Clean slots: 0 regressions in every arm. THIS IS THE IMPROVEMENT:
  20/20 → 0/20 under attack, at zero cost on clean ground.
- **W15 measured honestly**: habituation 0.10 + novelty gain 0.35 ALONE
  leaves the adversary undefeated (20/20 wrong) — novelty attention is
  an allocation policy, not a defense; it cannot outvote a confident
  majority. Also dormant on freshly-loaded static corpora (uniform
  novelty 1.0 cancels in the softmax; queries never write), and
  unreachable per-query (novelty_gain is config-level, not QueryTuning).
- **C. Mother's real corpus** (11 facet scenarios, 121 memories, 120
  bonds): W14 cap .35 trim 1 → top behavior STABLE 11/11 (hashes change
  on 10 — internal coalition redistribution, same answers); W18 margin
  .15 → answer-stable, mean activation 25.9 → 24.8 (mild efficiency);
  **W17 dedup .90 → top behavior FLIPS 4/11** — Mother's facet memories
  share IDENTICAL cues, so any cue-similarity dedup sees distinct
  memories as correlated copies and collapses whole facets to their
  strongest member. **W17 is a footgun on cue-degenerate corpora**;
  arming it requires value-aware or cue-enriched encodings. Fluency
  read on real data: **10/11 scenarios CONTESTED** (margin < 0.15), 1
  blank, 0 fluent — her facets genuinely disagree; the engine reports
  contested ground, not false confidence. W16's realtime metacognition
  is live and saying something true about the corpus.
- **Arming recommendation from measurement**: W14 on everywhere (free
  defense); W18 on where margins are wide (efficiency, answer-stable);
  W17 only on cue-distinct surfaces (claim surface: inert/no-op, safe;
  Mother: NO); W15 only where an observation loop runs (live traffic).
  130 tests green, fmt/clippy clean.

- **D49. MULTISCALE RENORMALIZATION admitted (#5)**: `renormalize(granularity)`
  coarse-grains the active memory while conserving evidence. Every pair of
  active traces at cue-similarity ≥ granularity merges into one GRAIN (seed
  = lowest id): evidence sums elementwise (the conserved quantity —
  minority dissent survives inside the merged distribution), support/mass
  keep the max (status, not substance), hits/contradictions add, novelty
  keeps the max. Members rest inactive inside the grain with a fine ledger
  (`grain_members` — what was removed, hash-visible); bonds touching a
  member REWIRE to the grain so connectivity survives re-description; a
  CONTRADICTION at the coarse scale SPLITS the grain (members re-activate
  with frozen evidence; the falsified coarse description dies with its
  post-merge earnings; bond rewiring is NOT undone — recorded asymmetry).
  Candidates never merge. Explicit operation only (D42 satisfied: cannot
  fire by accident); 1.01 = off, byte-identical, zero report.
- **Gates (tests/renormalization.rs, 5)**: evidence exactly conserved +
  majority answer preserved at unit scale (the anti-W17 gate); minority
  dissent survives (~25% share measured); contradiction splits the grain;
  bonds rewire and the grain stays reachable; off/validate/determinism.
- **The corpus-scale discovery (sweep, honest)**: on Mother at granularity
  1.0, 26 grains absorbed 87 traces (121 → 34 active, mean activations
  25.9 → 16.5 — a 3.5× working-set reduction with all evidence retained),
  but 7/13 top behaviors re-ranked (the protectee probe's top code moved
  P-001 → P-005, WITHIN the protectee family — family held). Mechanism:
  plain retrieval aggregates attention-weighted (reliability), the grain's
  merged distribution is evidence-mass-weighted — coarse-graining is NOT
  answer-invariant under attention-weighted aggregation. And compression
  did NOT manufacture confidence: coarse fluency stayed contested 12/13,
  fluent 0. **Verdict: renormalization stays OFF on Mother as-encoded**
  — her crowded facets are intended richness (the cue encoding is coarse,
  not the memories); the same verdict as W17, now with the rollback path
  and conservation the drop-lacked. Use where scale reduction is the goal
  and re-ranking is acceptable-visible.

- **D50. PARETO PRESERVATION admitted (#6)**: queries armed with
  `pareto_front` return the NON-DOMINATED FRONT of eligible traces over
  three objectives (reliability, support, novelty) — the set of
  contenders no other trace dominates (≥ on all axes, > on at least
  one). Where a single score collapses trade-offs into one winner, the
  front NAMES the tension: the habituated reliability champion and the
  fresh novelty contender both stand on it (gate: front size 2, values 1
  and 2). Fills part of the "tension as driver" creativity gap — tension
  is now visible structure, not just an abstention. **Pure metadata by
  construction**: the front is excluded from the response hash and never
  touches the mixture — gate proves claim, activations, and hash are
  byte-identical with the front on vs off, and the 60-claim benchmark
  holds 60/60 with it armed (all-on arm). Bounded by the eligible count
  (≤ capacity — the "unbounded front" risk is structurally capped).
  Deterministic order (reliability desc, id asc). O(n²) exact filter.
  Default off (empty front, zero cost). Gates: tests/pareto.rs (4) —
  including a brute-force independent verification that every front
  member is undominated and every excluded eligible trace is dominated.

- **D51. BET HEDGING admitted (#10)**: evolution's insurance policy, in
  the MEMORY layer (assertion path untouched). `hedge_rate` (0..=0.5,
  default 0 = off): the attention tail — the lowest-standing fifth of
  COMMITTED traces, reliability-ranked, ties by id — decays at
  (1 − hedge_rate) speed and KEEPS its ttl. Current losers persist as
  diversity insurance: when the regime flips, the alternative hypothesis
  is still there to answer instead of being re-learned from scratch.
  Measured (gate): after a long quiet period the unhedged engine has
  forgotten the lightly-supported alternative and ABSTAINS; the hedged
  engine still answers — and when evidence returns its answering trace
  carries ~3× the accumulated evidence of the unhedged fresh
  nucleation. **Hedge delays, never cancels**: a falsified (contradicted)
  trace still crosses the hard expiry floors — measured dying ~6 idle
  steps unhedged vs ~70 hedged, dead in both. Candidates never hedge
  (KPR dissipation is law); the hard support/mass floors are absolute.
  Claim benchmark with hedge armed (all-on): 60/60, 0 false — no cost on
  clean ground. Gates: tests/bethedge.rs (3). Fixture lessons recorded:
  contradictions must run while the victim still binds (revise refreshes
  ttl; a dead trace can only be nucleated onto), and single-member
  mixture weights read 1.0 — retention is measured in accumulated
  evidence, not claim weight.

- **D52. STIGMERGY admitted (#14)**: worlds leave bounded work-traces for
  other worlds. Bonds carry a `pheromone` level (0.0 default); a MARKING
  query (`query_marking` — explicit action, &mut self) deposits
  `stigmergy_rate` on every edge its expansion actually walked, each bond
  capped at `pheromone_cap` (4.0); evaporation multiplies every bond by
  `pheromone_decay` (0.90) on the decay clock — trails fade with time, so
  no route becomes a permanent rut unless traffic renews it; retrieval
  routing reads `stigmergy_gain × pheromone` as a bonus on top of the
  echo bonus. **The routing-not-evidence law, machine-checked**: after 50
  marking queries every TRACE is byte-identical to a plain-query control
  (gate asserts `traces() == traces()`) — pheromone may steer WHICH paths
  are walked and how strongly members weigh, never what any trace holds.
  **Stage-9 holds**: plain queries never deposit (query_tuned is &self;
  gate proves gain-on == gain-off hashes when no marking ran). **The
  visibility law**: pheromone rides in the bond line of the state hash,
  and a new public `state_digest()` lets auditors see routing bias move
  without waiting for an observe — gate asserts the digest moved while
  traces stayed byte-identical. **Coordination payoff measured**: a later
  probe through a marked route weighs the far (hop-2) member HIGHER than
  the same probe through an unmarked route; after evaporation (60 idle
  ticks) the advantage fades; deposits saturate at the cap (weight growth
  stops). All knobs default off/neutral; zero stays exactly zero, so
  un-armed engines are byte-identical. Gates: tests/stigmergy.rs (5).
  147 tests green, clippy clean.

- **D53. CATALYTIC ROUTING / EXAPTATION — rejected as a new mechanism,
  recorded as a verified mapping (#16)**: the admission protocol's
  novelty bar (contribute something no existing primitive expresses)
  FAILS for this candidate, and the rejection is itself a result with
  receipts (tests/exaptation_mapping.rs, 3 gates):
  - "Influence pathways without becoming evidence" IS stigmergy's
    routing-not-evidence law (D52) — re-proven in the recruitment
    geometry: marked routes strengthen a member's recruitment weight
    while every trace stays byte-identical.
  - "Recruit old structure for new use" IS echo-bond expansion + D38
    provenance: a memory observed only in facet A joins the composition
    of a question posed in facet B (cited, weight ≥ 0.10), its evidence
    votes through routing, nothing is copied into the asking facet, and
    no trace moves. Honest semantic discovered along the way: a bridged
    member can JOIN an answer but never OUTVOTE the asking facet's
    direct anchor (attention math: T·(sim + ln rel) — a sim-0 bridge
    cannot beat a sim-1 anchor). Recruitment supplements; it does not
    override — which is also why the draft "catalyst filter" mechanism
    (drop low-sim hubs from the mixture) is rejected: it cannot
    distinguish a hub from the bridged answer without an oracle. Both
    would filter identically.
  - The "hidden routing bias" risk is barred by construction: the
    recruited member rides in the activation set and the response hash
    (gate: bridged vs unbridged worlds hash differently).
  The queue's remaining entries (#7, #8, #4, #9, #13, #15) stay queued;
  the control family (#3, #11, #12) stays deferred per D40.

- **The five-primitive run is COMPLETE (2026-08-25)**: #5
  renormalization (D49), #6 Pareto preservation (D50), #10 bet hedging
  (D51), #14 stigmergy (D52), #16 rejected-as-mapped (D53). 150 tests
  green, fmt/clippy clean. Every mechanism defaults-off; the with/
  without law (D42) holds throughout; Mother's arming guidance updated
  by measurement (D48/D49).

- **D54. PRECISION-WEIGHTED PREDICTION ERROR admitted (#7)**: revision
  strength scales with the SOURCE's estimated precision — its
  (surprise-boosted) salience relative to the standing belief's
  reliability. `precision_gain` (0..=1, default 0 = off,
  byte-identical): weight w = strength/(strength + gain·reliability)
  where strength = effective_salience/2. On revise, the support cost,
  evidence dissolve, and dissent-vote add each scale by w (dissolve as
  factor^w). The error term was already present (PREDICT's surprise
  boost multiplies salience — surprised sources carry more precision);
  #7 adds the missing precision term. **Measured (gates)**: one
  0.25-salience ping against a 10-hit belief — armed retains >1.5× the
  standing evidence the control keeps, with a small nonzero dissent
  (heard, not obeyed); 3 full-salience (2.0) contradictions flip even a
  12-hit maximal belief (the escape hatch — precision never locks a
  belief); consistent 0.4-salience pings flip a moderate belief by
  ~10-12 vs the control's ~2-3 (resistance is real AND finite — bounded
  skepticism, no immunity). **Do-no-harm**: the 60-claim benchmark holds
  60/60, 0 false, 12 abstain with gain 1.0 armed in the all-on arm —
  the corpus correction (salience 1.0, last-writer each interleaved
  round) still revises. O10 stands at unit source precision (gain 0);
  under arming, a sub-precision ping is classified as noise, not a real
  contradiction — that classification is the mechanism. 154 tests
  green, clippy clean. Gates: tests/precision.rs (4).

- **D55. INFORMATION BOTTLENECK admitted (#8)**: `compress_evidence(dust)`
  — an explicit compression operation that prunes every evidence entry
  whose SHARE of its trace's total is below `dust` (the dissent dust
  weak pings accumulate; D54 makes those dissents small by design, #8
  removes the ones that never mattered). **The argmax is provably
  retained**: if the argmax entry were below dust×total, every entry
  would be, so total < value_count·dust·total — impossible while
  dust ≤ 1/(2·value_count), which validation enforces fail-closed. The
  report accounts removed mass EXACTLY (gate: loss == mass_removed to
  1e-12). Gates: argmax never pruned + dust cleared on a real dusty
  corpus (built via D54 weak pings — the two primitives pair: precision
  weighting casts small dissents, the bottleneck sweeps the ones below
  relevance); answers keep value AND fluency shape (the bottleneck
  constraint — compression without losing target-predictive structure);
  off/validation/determinism (dust 0 = byte-identical no-op). Claim
  benchmark: 60/60, 0 false with compression in the all-on arm.
  Relevance is share-relative by design (Tishby's target-dependence):
  the same absolute dissent mass survives on a poor trace and dies on a
  rich one.

- **D56. CAUSAL EVENT ORDERING admitted (#4)**: observations can carry a
  CAUSAL ORIGIN (`observe_with_origin`, origin 0 = unattributed =
  byte-identical to the default stream, hash-proven). Same-origin (or
  unattributed) revisions supersede exactly as before — the corpus
  correction is untouched. Two DIFFERENT explicit origins writing the
  same trace are CONCURRENT writers: the later arrival never dissolves
  the standing belief (no silent last-writer-wins); its dissent joins at
  precision weight (#7 composes: the concurrent vote is precision-scaled)
  and BOTH masses stay live — measured: the standing origin keeps ≥30%
  share after eight concurrent writes against five. **The money gate —
  permutation invariance**: swapping the arrival order of the two origins
  converges to the mirrored distribution (shares equal to 1e-6) and the
  QUERY answer follows the evidence mass, not the arrival order. Arrival
  order is no longer causally load-bearing where origins are attributed.
  Agreement is agreement regardless of origin: cross-origin
  confirmations reinforce and re-attribute. Traces carry `origin` in
  the state line (visible). Fixture honesty note: a python patch silently
  no-op'd on first attempt (whitespace mismatch) and the concurrency
  gate caught it immediately from the measured 0.55² dissolve residue —
  the gate did exactly what gates are for. Gates: tests/causal.rs (4).

- **D57. CRDT CONVERGENCE admitted (#9)**: `engine.merge(other)` — the
  engine is now composable (worlds/replicas unite deterministically).
  Join-semilattice semantics in three passes: (1) ANCESTOR JOIN — same
  id AND same cue = one memory diverged by replication: counters and
  evidence join elementwise MAX (G-Counter), tier joins to the more
  protected, active joins OR; a same-id DIFFERENT-cue collision is an
  independent-world artifact, never fused — it renumbers as fresh
  content. (2) FRESH ABSORB — other-side traces renumber into this
  engine's id space; exact-content duplicates collapse to the existing
  id (this is what makes re-merge idempotent — measured: second merge
  absorbs 0, dedupes 1). (3) BONDS — OR-set dedupe by endpoint pair,
  pheromone joins MAX; self-loops from remapping drop. Replicated
  counters join max; exploration joins to the cooler temperature.
  **Convergence is CANONICAL, not ordinal** — ids are engine-local
  ordinals, so A.merge(B) and B.merge(A) converge content-equal (gates:
  commutative, associative, idempotent by canonical projection) and
  answer queries identically. **Convergence ≠ truth** (the risk column,
  gated): two worlds asserting different values merge to CONTESTED
  ground — both masses survive the union, fluency reads Contested, the
  merge manufactures no winner. Fail-closed on cue-dim/value-count
  mismatch. CognitiveEngine is now Clone (replica forking). Gates:
  tests/crdt.rs (5).

- **D58. LATERAL INHIBITION admitted (#13)**: after selection (dedup)
  and before the robust cap, the WINNER softly suppresses its
  near-neighbors — each non-top member's weight ×(1 − g·sim²) against
  the top member's cue (quadratic: distinct voices keep their weight,
  only the redundant chorus around the winner fades) — then the mixture
  re-normalizes so share flows to distinct contributors. `lateral_
  inhibition` 0..=1, default 0 (off = byte-identical). Gates: the
  distinct contributor's share rises ≥1.5× as the chorus fades;
  mutually-distinct members untouched to 1e-9 (no artificial
  differences); a same-value chorus never changes the WINNER
  (suppression reshapes echo, not elections); determinism. **The
  interaction risk measured, not assumed**: W14's coalition defense
  relies on correlated copies voting, and #13 suppresses exactly those —
  the co-armed battery (W14 cap .35 + lateral .8) HOLDS 0/20 poisoned
  (the cap does the heavy lifting; the coalition members shrink
  EQUALLY, so their combined share still outvotes the capped
  adversary). Co-arming is safe on this geometry; fixture lesson: the
  distinct member must not already be the reliability winner or the
  chorus never holds the crown to suppress around. Gates:
  tests/lateral.rs (4).

- **D59. QUORUM SENSING admitted (#15)**: expensive deliberation (bond
  expansion) runs only on DISTRIBUTED support. `quorum_min` (0..=64,
  default 0 = off — always deliberate): the eligible set clusters at
  cue similarity 0.90 and counts DISTINCT supporters — correlated
  copies count ONCE, so correlated worlds cannot fake a quorum (the
  risk column, gated by construction: four identical-cue copies read
  ONE supporter and earn nothing). Meets the bar → the bridge fires;
  below it → the fast un-deliberated path, and the anchor still
  answers its own value (savings without answer cost). QueryOutcome
  carries a `QuorumReport { distinct_supporters, quorum_met }` so the
  report says exactly WHY deliberation ran or didn't. Composes with
  W18 satisficing (settledness gate) and the D40 watchlist: if
  chattering/route-flapping appears, quorum_min is the ready brake.
  Gates: tests/quorum.rs (4). **174 tests green, clippy clean — the
  sixteen-primitive queue is now fully processed**: built (#1 KPR, #2
  EIG, #5 renorm, #6 Pareto, #7 precision, #8 bottleneck, #9 CRDT,
  #10 hedging, #13 lateral, #14 stigmergy, #15 quorum), rejected-as-
  mapped (#16), deferred per D40 (#3, #11, #12 — the control family,
  awaiting the instability signs).

## D60 (2026-08-30, PINNED BEFORE CODE): the lightcone-core port — typed warrant from the original harness

> **D60 SUPERSEDED IN THIS COPY (Jesse, 2026-09-25; disposition amended same
> day).** This workspace now lives in
> `a-rag-atom`, whose scope strips Lightcone
> ("And yes strip lightcone"). D60's lightcone destination — the union north
> star (typed warrant enforcing geometry holding trained worlds) and the
> world-training anatomy — is OUT OF SCOPE and must not be built here. The
> NON-lightcone payload (D60a typed relations, D60b branch schemas, D60c
> creativity port) is **RETAINED by operator decision** ("keep it just in
> case it comes in handy, because the cognitive engine will need to connect
> to the harness") as candidate material for the future engine→harness
> connection. Execute no D60 stage as written — its gates assume the
> lightcone union; any use requires a re-scoped authorization under the
> a-rag-atom engine program.

Operator authorization: "Yes port it" — transfer Codex's reasoning
taxonomy + creativity engine from the original atom-harness (Python)
into this engine (Rust). Research-first pass completed: the source
artifacts are atom_philosophy_world.py (44 typed reasoning branches,
each a 5-step typed schema with signed step directions and typed
epistemic relations, hash-bound overlay, SEP-sourced),
atom_creativity_engine.py (7 first-principles with invariants:
generation-selection, conceptual blending, constraint-based
creativity, novelty-value balance, problem framing, tension
transformation, memory recombination), atom_integrated_creativity_
engine.py (lateral-inhibition routing — already ported as #13), and
the 8 stress families (atom_creativity_stress_benchmark.py).

Three-stage admission, defaults off, gates before each:

- **D60a — typed relations on bonds.** Bonds gain a RelationKind
  (supports, enables, clarifies, grounds, qualifies, corroborates,
  defeats, contradicts, undercuts, counterexample) with signed
  warrant effects; canonical hash over the typed overlay (the
  original lightcone's hash-bound discipline). Gate: existing bond
  behavior at untyped default is bit-identical (174 tests unchanged);
  new tests: sign propagation (defeat/undercut reduce warrant,
  support/grounds raise it), hash stability.
- **D60b — the 44 reasoning-branch schemas as data.** Ported as a
  typed table (branch → 5 steps, directions, relations, sources).
  NOT a new mechanism: schemas configure D60a relations over
  composition paths; deduction = strict chain warrant, etc.
  Gate: schema table validates (typed, hash-stable, complete vs
  source); held-out multi-step composition through branch schemas
  beats a shuffled-schema control (the original's withheld-path test).
- **D60c — the CreativityEngine module.** 7 primitives with their
  invariants; blends record contributing spaces; hard constraints
  non-tradeable; memory supplies candidates never truth. Admission
  tests = the 8 stress families ported (novelty_trap,
  constraint_poisoning, unsatisfiable_constraints included).
  Gate: pass ≥ the harness's own thresholds on all 8; invariant
  violations fail closed.

Follow-up (separate entry): mode-selective steering — the membrane
selects reasoning branch by type; re-run the E-series. Nothing is
hardcoded permanently until D60a-c gates pass.

### D60 correction (operator, 2026-08-30, primary source)

The ORIGINAL lightcone is GEOMETRIC — a causal cone, hashless. The
typed-graph-with-hashes found in the atom-harness is Codex's
implementation layer for lightcone knowledge, not the original
concept. Consequences: (1) the geometric original already exists in
this program as the universe-substrate (lattice light-cone, |Δx|+…≤Δ,
no hashes; Mother's carved memories = a trained world in geometric
form); (2) D60a-c remain the correct port OF THE HARNESS's portable
discipline (typed warrant, world-training anatomy: trained mechanisms
+ held-out compositions + honesty controls); (3) the destination
architecture is the UNION — typed warrant enforcing a geometry that
holds trained worlds — recorded as the D60 north star. Also verified:
"Lightcone worlds" are a named installation unit in the harness
(psychology 76-trained/114-held-out, coding 160/240, philosophy
176/264, algebra 40-branch; each with sealed benchmarks and
direction-inverted/untrained/ablation/equal-conflict/unsupported
controls) — the world-training anatomy is part of the port.

### D60 world inventory (operator-confirmed, complete, 2026-08-30)

The original lightcone's trained worlds — full set, superseding the
four-foundation list in the first correction note:
- SEVEN FORMAL DOMAINS (sciences + math as separate typed worlds):
  logic, algebra, geometry, calculus, chemistry, biology,
  information_theory (atom_formal_domains.py).
- FOUR PERMANENT FOUNDATIONS: psychology 76-trained/114-held-out,
  broad-coding 160/240, philosophy-reasoning 176/264 (+typed relation
  overlay), algebra-word (40 branches).
- GENERATIVE/COMPOSITIONAL: causal-world curriculum (7 root mechanics
  x 9 environmental axes), specialist worlds (8 domains, 2-edge
  isolation, held-out composition), massive causal world (Kaggle-
  trained/verified, transfer benchmark), tool-use world, language
  world, causal-knowledge world, universal knowledge phase.
- VERIFIED FROM ATOM_UNIVERSAL_KNOWLEDGE.md: the harness already
  implements "model = language rendering only; Atom owns routing,
  retrieval, graph formation, citations, claim identity, epistemic
  status." The questioner's surface-decoder framing is TRUE of the
  harness side; false of the newer Rust/membrane side (model still
  does inferential work there). Port scope D60b widens accordingly:
  the world-training ANATOMY (trained mechanisms + held-out
  compositions + honesty controls) applies to every world family,
  not just the four foundations.

### D60 addendum: the planner is in scope (verified 2026-08-30)

The harness's cognitive engine chooses reasoning per question via a
route-select-strategize pipeline (atom_cognitive_engine.py _plan):
every installed world is scored for the query; ALL worlds above
minimum_route_score are selected (up to max_active_worlds — one OR
more); strategy then follows the selection — multi_world_deliberation
(multiple worlds), decompositional_causal_chain (one world, no exact
law), fast_causal_recall (one world, exact law), active_world_search
(no world qualifies — refuses to bluff). Queries carry mode +
allowed_domains + transfer policy inside the canonical signature.
D60b therefore ports the branch schemas AND this planner layer;
selection is scored routing + strategy choice (the creativity engine
separately uses lateral inhibition — two selectors, two subsystems).

## D61 (2026-08-30): bond_expansion_depth cap raised 4 -> 8

Operator: "short term thermodynamic memory ... limited to up to 4
hops, find a solution to increase the hops" (top-2-per-hop already
failed per operator). The cap raise is gated: config validation 0..=8,
QueryTuning clamp min(8), persona floor assertion updated, and a NEW
gate test — depth-8 nine-fact chain crossing (tests/bonds.rs). All
engine tests green; defaults unchanged (default depth stays 2).
The compound wall also includes the 1/hop echo decay (argmax) and
field routing margins — engine reach is fixed; argmax verdict and the
lane-per-hop protocol results live in substrate-free-energy
RESULTS.md (hops section). Protocol lesson recorded there: two failed
runs were the memory running outside its validated protocol (missing
decay rates + conservation budget), the same class as E15.

## D62 (2026-09-27): a function may be SPLIT across positions; the thin policy that joins the crates

Operator, verbatim: *"a function can be broken into two or more, because
sometimes different positions give different behaviors that eaxh excels at,
so breaking them up to get the wanted bejavior among two or more is
justifiable, and as we found out, thin policy, rules, are needed to get more
than one create to work together"*.

Two clauses, both already measured in this project:

1. **Split placement is justified.** One function may be broken into two or
   more primitives placed at different positions, because a given position
   yields different behavior and each piece may excel where the whole does
   not. Our own evidence that POSITION decides: D61's readout placement
   ({S,R,A} order sweep — phantom mass **0.0** for SRA vs **4,784,611,219.8**
   for SAR, BUILD-26); BUILD-2's R5 (InjectThenMeasure collapses to 1 where
   MeasureThenInject reaches |H| = 10 — the same three primitives, a
   different capability); BUILD-1's {S superposition, R destructive readout,
   A accumulation} are themselves three pieces of one interference cycle.
   The split is governed by the same discipline as everything else: each
   piece is a genuine mechanism (not a renamed whole), matched controls run,
   and the composed result is promoted only on a measured improvement.

2. **Thin policy joins the crates.** More than one crate working together
   requires a thin, declared policy layer — our measured instance is the
   BUILD-13 boundary: **one** policy surface, 85 lines against 6,253 engine
   lines = **1.36%**, select-only (its report is byte-identical to a direct
   invocation), fail-closed on unknown shapes, content-blind, and extended
   only additively afterwards (evidence shape, declared chains). The operator's
   reasons, on record: LINE COUNT and SECURITY — authored decision surfaces
   are attack surfaces, so the policy stays thin, never computes, and never
   inspects content.

**Consequence for the chaos work (BUILD-32):** the end-position verdicts
stand — chaos as a single whole function at the END was inert on Stack A and
difference-without-benefit on Stack B. D62 supplies the next legitimate move
rather than a reversal: **split the mechanism and place the pieces where each
excels** (e.g. the logistic's expansion and folding as separate stages; the
Hénon stretch term and its fold as separate stages), under Rule 5's matched
controls, with the same promotion rule — and with the honest note that this
extends beyond the operator's earlier "end of each stack" narrowing, so it
runs when he asks for it.
