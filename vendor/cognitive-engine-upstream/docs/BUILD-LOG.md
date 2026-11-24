# Build log — a-rag-atom

Append-only. One entry per build (and per pre-BUILD experiment), written as
part of the build's receipts stage, from that build's own outcome section —
never re-narrated later from memory. Detail lives in each build's doc; this
log carries the short version plus **the lesson**, so recurring mistake
classes become visible across builds. The habit is carried by the
`atom-build-protocol` skill.

Fields per entry: **Gates** · **Corrections / findings** · **Lesson** ·
**Commits**.

---

## E1 — perception admission (2026-09-25)

- **Gates:** 4/4.
- **Corrections / findings:** gate 2 failed first run — the one-vs-rest detector metric counted true negatives; corrected to naming accuracy (best single-dimension namer 6/9). Kept on record.
- **Lesson:** precommit the metric to the behavior VERB (detector vs namer), not to a convenient aggregate.
- **Commits:** 5a2e628.

## E1c — the chaos arm (2026-09-25)

- **Gates:** 2/2 after correction.
- **Corrections / findings:** predicted ±2 neutrality under chaos; measured HARM (27 vs 39 of 45), so the gate became "no benefit (harm allowed)". Zero mislabels at every amplitude — chaos degrades perception into abstention, never into wrong naming (fail-safe).
- **Lesson:** predictions about ABSENCE need the same care as presence — and fail-safe is not the same as benefit.
- **Commits:** 5a2e628.

## E1-chem — the catalysis arm (2026-09-25)

- **Gates:** 4/4.
- **Corrections / findings:** CH1 expected new 2-id evidence; the engine returned the active hypothesis with its founding evidence — the ENGINE was right, the assertion was wrong.
- **Lesson:** when a hand-expectation disagrees with the engine, check the engine's semantics first; several first-run failures were my accounting, never the code.
- **Commits:** 5a2e628, 1cbe1c2.

## E1R — emergent perception (2026-09-25)

- **Gates:** 4/4; purity 1.000 with the zero-typed-content resonance bank.
- **Corrections / findings:** the arrangement sweep measured 0.667 vs 1.000 — one arrangement carried the behavior, the other did not.
- **Lesson:** a quantum-SHAPED representation is not enough; the base must be load-bearing in the composition.
- **Commits:** 8bf9a2e.

## E1R-2 — SWAP-test naming (2026-09-25)

- **Gates:** 5/5.
- **Corrections / findings:** a REAL circuit bug — `cswap` built as three same-control controlled-X gates; same-control gates commute (X_a·X_b·X_a = X_b), so the swap collapsed to a single flip and the SWAP test read 0.305 fidelity on identical states (physics: ~1.0). Fixed as a direct amplitude permutation.
- **Lesson:** verify a primitive's algebra against a case whose answer is known; staged state dumps found what reasoning had missed.
- **Commits:** df08bd6.

## E1R-3 — the Born-evidence stack (2026-09-25)

- **Gates:** 5/5.
- **Corrections / findings:** coupling resolved as CHARACTER IMPOSED with rates on closed forms; M reads without re-preparation are ONE effective sample (all-or-nothing); erasure is not observation currency; load-bearing inventory recorded (catalysis and chaos-on-iid inert).
- **Lesson:** "it works" has a ceiling — measured equals an independently derived closed form, with no external oracle.
- **Commits:** 1432cb8.

## E1R-4 — the full combination sweep (2026-09-25)

- **Gates:** 4/4.
- **Corrections / findings:** all 48 combinations swept (chaos × catalysis × floor × erasure policy); the champion configuration measured (strong 99.5%, noise 0.2%).
- **Lesson:** sweep ALL combinations under matched controls — the first working order is never presumed best.
- **Commits:** becb9ee.

## BUILD-1 — synthetic emergence from quantum functions (2026-09-25)

- **Gates:** 6/6 (after three harness-side iterations on G1).
- **Corrections / findings:** G3 physics error corrected (washout → memorylessness: phantom fringes 0.0 with destructive readout vs 4.8e9 without); the corpus bug (one regime per window = single plane wave = flat modulus) fixed by co-presenting regimes per epoch; G1 closed by N=20 + the interface convention k = cycles/site, derived analytically first (leakage ~1e-19).
- **Lesson:** a single plane wave is structureless — a measurement observes a SCENE; interface conventions must be derived in closed form before landing.
- **Commits:** fa6ccc9, b28cc5e, 78302cd, 0f70af2.

## BUILD-2 — reasoning as interference closure (2026-09-25)

- **Gates:** 5/5.
- **Corrections / findings:** closure support = 10 = |H| exactly (hand-derived before the run); order sweep measured the decisive finding — InjectThenMeasure collapses to 1 (squaring the measurement creates the relation products; coherent-amplitude feedback dies).
- **Lesson:** the ORDER decides whether derivation chains at all; feedback forms are load-bearing, not interchangeable.
- **Commits:** 3a0d152.

## BUILD-3 — perception as emergent identification (2026-09-25)

- **Gates:** 6/6.
- **Corrections / findings:** two hand-accounting corrections (the engine measured correctly throughout); recognition is an exact integer support delta with no threshold anywhere; the feedback-closure variant (c) named with its closed form, deferred.
- **Lesson:** exact integer decisions remove the need for thresholds — and hand arithmetic is the least reliable part of an experiment.
- **Commits:** e437132.

## BUILD-4 — preference (2026-09-25)

- **Gates:** 4/4.
- **Corrections / findings:** the operator arm's precommitted prediction failed: BOTH operators reach the argmax plateau at 10 steps — the earlier "multi-site equilibrium" reading was against my own wrong belief that k=1. Corrected on record.
- **Lesson:** check the closed form's assumptions (what k IS) before predicting which arm fails.
- **Commits:** 3e5c7e7.

## BUILD-5 — action and consequence (2026-09-25)

- **Gates:** 4/4.
- **Corrections / findings:** exact learning through action (support == recorded-action closed form, every policy); the distinctive contrast measured — 50 passive presentations of the world's wave teach NOTHING, while the action loop learns its 81-bin relational structure.
- **Lesson:** consequence, not observation, is what puts structure into the field — and a closed form computed from the engine's own record is the honest comparator.
- **Commits:** 3e5c7e7.

## BUILD-6 — generation (2026-09-25)

- **Gates:** 3/3.
- **Corrections / findings:** TWO corrected implementations on record — a fixed-point step (phase-blind reweight recreated the same wave: 2 distinct frequencies per 64 epochs) and the readout twice (wrapped mean had boundary contamination; DFT interpolation too noisy → the multi-lag phase unwrap won).
- **Lesson:** incommensurate shear needs phase-PRESERVING reweight (ψ·|ψ|); when a readout misbehaves, change the estimator, not the mechanism.
- **Commits:** 3e5c7e7.

## BUILD-7 — selection: measured NEGATIVE (2026-09-25)

- **Gates:** 1/1 (the test asserts the negative).
- **Corrections / findings:** selection in processing did NOT emerge (mean |a−W| 0.570→0.541 vs 0.499 baseline); mechanism identified — accumulation is not ranking; the composite candidate named, not built.
- **Lesson:** a negative is a finding when the mechanism is named — "accumulation is not ranking" is what made BUILD-14 possible.
- **Commits:** 3e5c7e7.

## BUILD-8 — the geometric reasoning engine (2026-09-25)

- **Gates:** 4/4, first run.
- **Corrections / findings:** all closed forms pre-verified in Python; the geometric primitives ARE the substrate's native objects (translations = differences, inner product = metric, conjugation = reflection, unitaries = rotations).
- **Lesson:** when the substrate's own algebra IS the domain's algebra, the engine needs no translation layer — and Python-first showed it.
- **Commits:** dc1123b.

## BUILD-9 — spatial reasoning (2026-09-25)

- **Gates:** 5/5.
- **Corrections / findings:** one first-run mapping error kept on record (my 180° index map); the substrate was exact throughout; chirality measured (+12 rotations, −12 reflection) and frame equivariance at 3e-14.
- **Lesson:** SSS congruence is blind to handedness — signed, oriented structure is what identifies WHICH transformation occurred.
- **Commits:** b52c3bd.

## BUILD-10 — the reasoning-type suite (2026-09-25)

- **Gates:** 6/6.
- **Corrections / findings:** the analogical first version failed on record (side lengths cannot identify which rotation; signed difference vectors can); defeasibility measured to COST memorylessness; three retractors measured and rejected.
- **Lesson:** the trade is the result — record what a capability costs, not only that it appeared.
- **Commits:** 9fc7ce9.

## BUILD-11 — generative knowledge (2026-09-25)

- **Gates:** 5/5.
- **Corrections / findings:** 997 × 638 = 636086 generated exactly from two stored blocks (two coprime registers fold and reconstruct); sufficiency as exact closure membership; knowledge economy measured (2 relations → 10 answers).
- **Lesson:** facts are content, procedures are not content — generators must be intrinsic, and sufficiency must be a readout.
- **Commits:** f55d297.

## BUILD-12 — wiring the engines to the harness (2026-09-25)

- **Gates:** 4/4.
- **Corrections / findings:** a copy-paste closed-form error (asserted 6 bins for a collinear scene; correct is 4) corrected before landing; the ordering-validated composite measured; content-blindness measured (derivable and non-derivable tasks take the identical route).
- **Lesson:** the harness is a FIXED composite — verdicts are outputs, never selectors; and harness fixtures deserve the same Python-first treatment as engines.
- **Commits:** 59394f9.

## BUILD-13 — the boundary policy layer (2026-09-25)

- **Gates:** 5/5.
- **Corrections / findings:** policy touches nothing (byte-identical outcomes to direct invocation); fail-closed refusal; line economy measured — 85 policy lines vs 6,253 engine lines = 1.36%.
- **Lesson:** policy at ONE thin boundary, select-only, fail-closed — authored decision surfaces are attack surfaces and line-count liabilities.
- **Commits:** 4b0d566.

## BUILD-14 — selection: relaxation over consequence-accumulated structure (2026-09-26)

- **Gates:** 6/6, first run.
- **Corrections / findings:** participation == k == |K| == N²/|H(D)| exactly; the BUILD-4 law transferred to consequence-learned data (cross-check: k=40 for BUILD-4's own scene); the composite's selection is evidence-CONSISTENCY, not world-proximity (the BUILD-7 |a−W| metric recorded at 0.854, not gated).
- **Lesson:** a named composite can be built cheaply once its mechanism is named; and be explicit about which metric the new mechanism does NOT move.
- **Commits:** 7233bda (precommit) → eb30086 (build) → 22fca7e (vocabulary).

## BUILD-15 — the thermodynamic scratch pad (2026-09-26)

- **Gates:** 7/7, first run.
- **Corrections / findings:** harmonic self-scaling dissipation gives mass fraction exactly 1/(T+1)² with no authored rate; six scenes through one pad reuse exactly (supports [6,3,7,3,7,2], mass cycling 1200/800) vs contamination [6,12,36,52,84,107] without dissipation; prediction FAILED on record — T_full/2 does not always contaminate (visibility depends on bin powers): the derived budget is sufficient, not tight.
- **Lesson:** empty and degenerate states are where hand-predictions fail; gate the RELATION and record the measured nuance, never the assumed magnitude.
- **Commits:** d2a4ea6 (precommit) → 395dc0f (build).

## BUILD-16 — the harness runs on the pad (2026-09-26)

- **Gates:** 6/6.
- **Corrections / findings:** THE WIRING FOUND A REAL BUG — clearing by dissipation at the pad's own derived budget (31622 ticks) left the previous task visible (scene_support 6 vs fresh 2) because the harness's readout thresholds at an ABSOLUTE 1e-9 while that budget serves a RELATIVE readout; the consumer's requirement is ~37.9M ticks (~1e10 site-ops per boundary). Shipped fix: the reset arm at the boundary (bit-identical reuse, measured).
- **Lesson:** clearing budgets are CONSUMER-scoped; and wiring a component into the path is what makes its bugs findable — an unfailable component never fails.
- **Commits:** 8fdbc80 (precommit) → bffa90a (build).

## BUILD-17 — the block library (2026-09-26)

- **Gates:** 6/6, first run.
- **Corrections / findings:** content-only crate with ZERO dependencies (it cannot compute) and no engine dependency (content-independence); engine-derived product 636086 from library content (alt content 998 → 636724); adding a domain is data; prediction corrected on record — `derivable_count` of nothing is 1 (the trivial relation, DC), not 0.
- **Lesson:** a library that cannot compute is the strongest form of "facts, never procedures" — make the boundary structural, then the audit is a compile-time fact.
- **Commits:** 0aef09b (precommit) → c673041 (build).

## BUILD-18 — content classes: blocks, world facts, personal (2026-09-26)

- **Gates:** 6/6, first run.
- **Corrections / findings:** separate stores, one schema, one ENFORCED class; an adversarially wrong personal claim changes NO derivation answer (636086 before and after) while explicit acquisition yields 637362 by design; a Context partition is refused by the derivation accessor (fail-closed).
- **Lesson:** separation by TRUST CLASS beats separation by folder — and it is the class check at every use, not the crate boundary, that protects the blocks.
- **Commits:** 921340a (precommit) → 946f571 (build).

## BUILD-19 — the reserved store (2026-09-26)

- **Gates:** 3/3, first run.
- **Corrections / findings:** a declared, empty, fail-closed home (class `Context`, status stated); it cannot poison (measured), and adoption is a class flip alone (Context → Derivation derives 7×3 = 21, no new code).
- **Lesson:** reserve structure by decision and keep it fail-closed; an unused store is a cheap cost, a layout change later is not.
- **Commits:** f638e82 (precommit) → 16299ea (build).

## BUILD-20 — the build log (2026-09-26)

- **Gates:** 3/3, first run.
- **Corrections / findings:** the log is a gated artifact — `include_str!` at compile time, so deleting it breaks the build, not just a convention; backfilled exactly from the status note and git history (19 builds + the E-series, receipts cited per entry); the habit is carried by `atom-build-protocol` step 7, so a project without a log gets one created as part of its receipts stage.
- **Lesson:** a lesson is only learnable if it is written where the NEXT build will look — per-build receipts are prose until they are collected into one place.
- **Commits:** 11018a9 (precommit) → 145edd5 (build; this entry is in it) — the log's first self-referential entry.

## BUILD-21 — the library at scale (2026-09-26)

- **Gates:** 5/5, first run.
- **Corrections / findings:** four NEW partitions appended (the recorded fixtures stay byte-identical — growth by addition, never by editing content earlier gates stand on), numbers verified in Python first: 13 new stored facts; 8 stored values → 16 derived products, each equal to plain arithmetic through the engine's CRT path; 4 stored relations → closure 50; the reach-out target (1,0) flips false→true on explicit acquisition and the closure doubles to exactly 100; hygiene gate (no duplicate blocks); the class boundary holds at scale.
- **Lesson:** the economy is only honest if the products are never stored — and duplicate facts would lie about it, so hygiene is a gate, not a nicety.
- **Commits:** a19c5e7 (precommit) → c533956 (build; this entry is in it).

## BUILD-22 — the lightcone adapter (2026-09-26)

- **Gates:** hermetic 4/4 (X1/X2/X3/X6) and live 2/2 (X4/X5) pass; the suite's only other failure is by design (hops, D61).
- **Corrections / findings:** authorization verbatim — Jesse: "Ok, lets try" (read-only, passive, subprocess adapter; one section only). The contract was verified live BEFORE the precommit: `verify` on ai-development-v0@0.2.0 confirms 48D coordinates / 4D observer shadow / `adapter_independent: true` / manifest_hash 1b8e898c…; `admit "activation function"` returns seed ai.activation, admitted [ai.activation, ai.layer, ai.root], two traversed edges at hop 1, `passive: true`, `holds_verdict: false`. Determinism measured: the same call twice returns the same receipt hash — equal to the committed fixture's hash — and a different direction returns a different one (the policy is in the receipt). Adversarial refusals: seven mutations, seven typed errors, no fallback. L4 (the build-log gate added in BUILD-20) failed this build before its log entry existed — the enforcement working on its first opportunity.
- **Lesson:** the lightcone's receipt already carries the discipline we keep hand-building (passive, holds_verdict, hash-bound provenance) — an adapter's job is to ASSERT those invariants on every payload rather than trust the source, and a schema change must fail closed rather than follow silently.
- **Commits:** 6831876 (precommit) → the commit that adds this entry.

## BUILD-23 — acquired evidence: the reach-out door fed by a real source (2026-09-26)

- **Gates:** hermetic 5/5 (E1–E4, E6) and live 1/1 (E5) pass.
- **Corrections / findings:** the seam is a trait the ENGINE owns (`evidence::{Evidence, EvidenceSource, EvidenceError}`), implemented by the adapter (bridge → engine, one-way); `Evidence` carries source id + receipt hash + counts and NO payload; `run_with_acquisition` asks the source exactly once, unconditionally, and reports the receipt — no stage consumes it. Measured: one call in both derivable and non-derivable tasks; derivation fields identical to a direct run; two different receipts change only the `acquired` field; a failing source yields a typed error and no report. LIVE: a real admission became evidence — `lightcone:ai-development-v0@0.2.0`, receipt `dc7a388f…`, nodes 3, edges 2 — exactly the BUILD-22 measurement, derivations unchanged. L4 failed the suite with "build docs without log entries: [23]" before this entry existed — second consecutive catch.
- **Lesson:** provenance and content are separable — an external source can be load-bearing for a report while being structurally unable to touch a derivation; declare the seam in the engine, implement it in the adapter, and assert the direction with manifest scans rather than promises.
- **Commits:** 1b8f79d (precommit) → the commit that adds this entry.

## BUILD-24 — pinned and exposed: the lightcone on the operator's path (2026-09-26)

- **Gates:** hermetic 5/5 (N1, B1–B3, C1) and live 3/3 (N2, E5 re-run, C2) pass; BUILD-13's five gates re-verified intact.
- **Corrections / findings:** the pin is REQUIRED on every lightcone source — an unpinned source cannot be constructed (the fix for "nothing pinned"); a receipt whose manifest_hash differs is a typed drift refusal naming both hashes (live: pinned 000… vs got 1b8e898c…). The boundary gained a declared evidence shape reachable only through `invoke_with_source`; all three refusal doors are fail-closed, and the source is never consulted on a refused path (measured: zero calls). The operator surface is live: `atom-cognitive-cli --lightcone-demo` with LIGHTCONE_CLI/PACK/PIN exits 0 and prints the record's receipt (dc7a388f…); with the environment unset it exits 2 naming the variables. L4 caught the missing log entry a THIRD consecutive time before this entry existed.
- **Lesson:** a pin turns "validated once" into "validated every time" — it changes nothing about the world and refuses the moment the world changes; and exposure across three surfaces (adapter, boundary, cli) stayed additive because every new door is a declared lookup plus a typed refusal.
- **Commits:** 6630b56 (precommit) → the commit that adds this entry.

## BUILD-25 — the closure variant of perception: BUILD-3's deferred Rule-2 arm (2026-09-26)

- **Gates:** 6/6, first run; every Python-verified prediction exact.
- **Corrections / findings:** the variant BUILD-3 named and deferred (c) is now MEASURED — each presentation's own BUILD-2 closure (its 40-step canonical protocol, matched controls by construction) unioned as BINS. Deltas [10, 0, 40, 0] with supports [10, 10, 50, 50] against the canonical [6, 0, 4, 0]: identical recognition decisions, coarser magnitudes — the measured trade of composing the two builds (subgroup structure bought; fine bin granularity and arithmetic cost given up: 40 closure steps per presentation). Cross-check inside the new composition: closure(triangle) = 10 = BUILD-2's measured |H| for the same scene, reproduced independently. The shared accessor refactor (`support()` delegating to `support_bins()`) left BUILD-2's and BUILD-3's gates green. BUILD-3's declared variant set is now COMPLETE.
- **Lesson:** a deferred variant closed two builds later costs little and pays twice — it discharges the Rule-2 debt AND reproduces the earlier build's closed form inside a new composition (the 10 verified itself twice).
- **Commits:** ec765b7 (precommit) → the commit that adds this entry.

## BUILD-26 — BUILD-1's own order sweep: the last Rule-2 debt (2026-09-26)

- **Gates:** 4/4, first run; the recorded magnitudes re-observed exactly.
- **Corrections / findings:** the foundational stack's primitive set is now EXPLICIT — S (coherent superposition with the epoch carrier), R (destructive Born readout), A (intensity accumulation) → 3! = 6 unique orders. Constructible: **SRA** (canonical `InterferenceField`) and **SAR** (no-reset `CoherentField`); untested and NAMED: RSA, RAS (readout before any field), ASR, ARS (accumulation before content). Matched controls: both arms over the identical 18 windows and the identical 9 phantom vectors (BUILD-1's G3 construction). Measured: phantom mass **0.0** (SRA) vs **4,784,611,219.8** (SAR) — the readout's POSITION decides. Per Rule 2 the space cannot be fully executed, so the untested orders are reported and the claim is bounded: "best observed among the EXECUTED orders; no global-best claim". No engine code changed — the two constructible orders ARE the existing arms.
- **Lesson:** the Rule-2 debt was dischargeable with zero new mechanism because the "ablation arms" of earlier builds WERE the alternative orders all along — enumerating the order space retroactively turns ablations into a sweep table, and the claim shrinks to what was actually executed.
- **Commits:** c81bab6 (precommit) → the commit that adds this entry.

## BUILD-27 — hierarchical composition: invoking declared stacks (2026-09-26)

- **Gates:** 6/6, first run; BUILD-13's five and BUILD-24's three boundary gates re-pass unchanged.
- **Corrections / findings:** the orchestrator clause's second verb is now built — a registered stack may INVOKE registered stacks, nested by declaration. `Chain` + `Registry::validate` (unknown names and cycles are construction errors) + `TaskShape::Chain(&'static str)` + `Request::Chain(Vec<Request>)` (count mismatch → Refused) + `Outcome::Chain(Vec<Outcome>)`. Depth is fixed by the declared DAG: no runtime budget, no branch on content. Measured: nested outcomes equal direct invocations field for field (the composition adds no computation); nesting is visible in the report (depth 2); refusals surface IN PLACE while the remaining stages still report; content-blindness holds at depth. Line economy: boundary.rs 118 → 218 (+100 for the whole composition surface = 3.09% of the 7,058 engine lines, inside the recorded <5% bound).
- **Lesson:** composition by DEPTH needs no routing and no budget knob — a validated acyclic table IS the bound, the report shows the nesting, and the only new authored surface is data.
- **Commits:** 2483b55 (precommit) → the commit that adds this entry.



## BUILD-28 — the causal-path primitive: arrows, sequences, bound traces (2026-09-26)

- **Gates:** 8/8 (C1, C2, C4–C7 + the Rule 2 enumeration arm) pass; BUILD-5's four and BUILD-14's six gates re-pass byte-untouched through the shared-law refactor.
- **Corrections / findings:** the flagged design question is answered by measurement, not argument. The loop now records its OWN arrows (`Action → Content` for the bins an epoch writes, `Content → Action` for the substrate an action was sampled from) through one shared implementation of BUILD-5's law (`difference_of` + `bins_of_difference`), so the closed form and the arrows cannot drift. Headline: `support(A) = support(mirror(A)) = 5` for A = [(1,1),(3,7)] against its mirror about the world's wave, while ALL FOUR bins name a different producer — **our own readout is direction-blind; the recorded arrows are not.** Walks are returned origin-first in traversal order; bounds are declared (`truncated` reported, never silently partial); unknown origins are typed refusals; receipts are canonical-form digests (walk order and recording order cannot change them). FOUR precommitted statements were wrong and are corrected on record: (i) Content includes the **DC bin**, which nothing produces and everything samples; (ii) content DOES have outgoing arrows — the arrows alternate Action → Content → Action, and the sharper truth is that the producer is never among the samplers and every sampled substrate was produced in a strictly earlier epoch; (iii) walks are origin-first, not causal-order; (iv) the arm's lists are destination-free with a lexicographic tie-break. Structural finding: a bin's substrate fan-out grows with the epochs it survives, so walk counts grow fast with the bound — the bound is load-bearing. L4 caught the missing log entry a FOURTH consecutive time.
- **Lesson:** the binding rung was never missing a mechanism — it was missing the ARROWS, and they were already being computed and thrown away by our own ± union. Recording what the run did (rather than reconstructing it) turns a support number into a past cone, and the passivity is type-level: every readout is `&self`, so traversing cannot change the run even when interleaved into it every epoch.
- **Commits:** beca33c (precommit) → the commit that adds this entry.

## BUILD-29 — derivation word-paths: the sequence of stored relations that produces a member (2026-09-26)

- **Gates:** 8/8 (W1–W8) pass; BUILD-28's eight gates re-pass through the one additive change (`digest` exposed); every precommitted number landed exactly.
- **Corrections / findings:** the closure answers *whether* (one membership bit and one count); the **word** answers *how* — the ordered sequence of stored relations whose composition lands on the member. The graph: one node per closure member, one directed edge per stored relation (`m → m + g`, so the edge names its generator), and a walk from the identity IS a word. Measured: **7 stored relation blocks → 4 distinct relations** (dedup gated); the graph **IS** the closure (membership agrees with `derivable` at all **400/400** cells, so the structure is a strict enrichment); the recorded closure numbers reproduce (10 / 50 / 100); **one membership bit, 140 derivations** — target `(14,16)` has exactly 140 distinct words, all of length 7 and none shorter, each replaying to the target; the **order is free and the path is not** (six permutations of `[(0,8),(6,0),(10,0)]` all reach `(16,8)`); bounds never read as absence (six hops → zero words while `shortest` still reports `Some(7)`); a non-member is definitive (zero words, `shortest == None`); an empty library is the primitive's typed refusal and its closure is the identity alone (BUILD-17's correction); the **reach-out acquires a route** — `(1,0)` goes from no words to `shortest == 1` and 972 words through seven hops, per-length `1, 0, 3, 24, 35, 146, 763`; the whole 50-member shortest table is asserted. TWO CORRECTIONS on record: a transcription slip in my own gate (asserted the shortest-length count as the total) and a real semantics fix the table caught — the identity's shortest word is the **empty** word (0 relations), not the 2-hop cycle my BFS first reported; the enumeration still lists the cycles (0/1/6/7 at lengths 1..4). The original `derivable`/`derivable_count` answers are unchanged. L4 caught the missing log entry a FIFTH consecutive time.
- **Lesson:** the degenerate member is where a hand-written specification slips — the identity needed an explicit "no relations applied" answer, exactly as BUILD-17's empty closure needed "the closure of nothing is 1"; and the same primitive that recorded the action loop's arrows computes a derivation's provenance with no modification, because a word IS a walk and the library's relations ARE differences.
- **Commits:** 5691c3f (precommit) → the commit that adds this entry.

## BUILD-30 — the provenance report: the derivation's words wired into the harness (2026-09-26)

- **Gates:** 9/9 (P1–P6, P8 engine-side; P7/P7b on the operator surface) pass on the first run; BUILD-12's four, BUILD-13's five, BUILD-16's pad gates, BUILD-23's five, BUILD-24's three and BUILD-29's eight re-pass — with ONE recorded gate edit (H6, below).
- **Corrections / findings:** Jesse's BUILD-16 steer ("wire it in, cause its the only way to find bugs in it moving forward") applied to the two readouts nothing consumed. `HarnessReport` gained `provenance: Option<Words>` — the same reported-never-consumed discipline as BUILD-23's `acquired` — fed by a new entry point `harness::run_with_provenance(task, hops)` that builds the word graph over **the task's OWN declared blocks** and enumerates to **the caller's declared bound**, and exposed through the declared shape `TaskShape::SceneQuestionWithProvenance` + `Request::SceneWithProvenance { task, hops }`, plus the appended chain `provenance_then_product` and the operator surface `--provenance-demo`. Measured: the wiring is **inert** (every recorded field identical with and without it; two different bounds give ONE report — the provenance is the only difference); the provenance is **exact** (the report's digest equals the direct readout's; 140 words at seven hops, 6 at three for (16,8), 17 with per-length `1,0,3,13` for (0,8)); the bound is **the caller's and declared** (six hops → 0 words while `shortest` still reports `Some(7)` and `truncated` is true); the **two absences are distinguished** (no structure → the primitive's typed refusal and NO report; a non-member → a report with zero words and `shortest == None`); mismatches **refuse before any run**; inside a chain the nested stage equals its direct invocation field for field; the CLI's printed digest equals the readout computed independently in the test. **The surface audit paid off:** `Boundary::invoke`'s match is exhaustive with no wildcard, so extending the enum was compile-checked — the compiler forced the new arm in, and nothing fell through. **The ONE recorded gate edit:** H6 asserted `CHAINS.len() == 2`; the table grew to three by declaration, so the count moved **in the open** (reason in the gate's own comment), the gate's substance unchanged — and P8's "BUILD-27's six gates untouched" claim is corrected on record: five were, H6's cardinality was not. Line economy: harness.rs 170 → 194, boundary.rs 218 → 247 (3.18% of 7,771 engine lines, inside the <5% bound), CLI 651 → 726. L4 caught the missing log entry a SIXTH consecutive time.
- **Lesson:** a report field is the cheapest place to find a wiring bug — the provenance changed nothing (proving no stage consumes it) while the digest equality proved the wire carries the real readout and not a copy; and extending a recorded enum is safe exactly when its matches carry no wildcard, which the surface audit establishes BEFORE the edit rather than after.
- **Commits:** f1e1e1e (precommit) → the commit that adds this entry.

## BUILD-31 — the inverse readouts: gcd, divisibility, quotient and inverse as measurements (2026-09-26)

- **Gates:** 7/7 (D1–D7) pass on the first run; every precommitted number exact; BUILD-29's eight gates re-pass byte-identically through the performance edit.
- **Corrections / findings:** the operations the sufficiency inventory recorded as MISSING — division, divisibility, the modular inverse, gcd — are **readings** of the structures already owned, not algorithms: a **counted subgroup order** (`gcd_of(a,b) = b / order(⟨a⟩ in Z_b)`), an **equality** (`divides`), and a **shortest word length** (`quotient_within` = the least k with k·b ≡ t, `modular_inverse` = the same at t = 1). No Euclid, no extended Euclid, no division loop exists anywhere in the new code. Measured: `gcd(12,20) = 4`, `gcd(15,20) = 5`, `gcd(638,2^20) = 2`, `gcd(0,20) = 20`, `gcd_of(a,0) = None` (fail-closed); reachability equals `gcd(b,m) | t` on all eight declared cases; `997` at `2^20` gives `636086 → 638` (BUILD-11's recorded step count recovered from the structure, the word's relation count equal to the quotient and its endpoint the target); `997^-1 mod 2^20 = 950765`, replaying to 1, while `12` at `64` gives `None` (no inverse exists); the **mirror arm** — generator `b` vs `-b` reaches the SAME subgroup (order 1048576 and 16 identical) while the least k follows **k' = order − k** (638 → 1047938; 3 → 13), the same membership with a different word, which is BUILD-28/29's headline shape reappearing inside arithmetic; the **cross-check with the march's own readout** (299100 → 300 steps) including the folding boundary named (`-412490` folded, residue 636086 → 638). Register scale: **2^20 members + 1,048,576 edges in 659 ms**, reads in 234 µs and 464 ms — and the set-based membership edit was load-bearing, not cosmetic (the previous linear scan would have been ~10^12 operations at this scale). The honest boundary restated: every readout is MODULAR within a declared register, the unbounded range is only reached through the CRT pair, `gcd_of(a,0)` refuses rather than reporting a convention, and `divides(a,0)` claims nothing. L4 caught the missing log entry a SEVENTH consecutive time.
- **Lesson:** the ladder's missing rungs were not missing mechanisms — the same closure and the same words, read as an order, an equality and a length, answer division, divisibility, the modular inverse and gcd; a capability can be extended without authoring a single procedure, and the only engineering the rung needed was making the structure hold a million members.
- **Commits:** 2143f6c (precommit) → the commit that adds this entry.

## BUILD-32 — the end-of-stack chaos arm: add it, measure it, reject it unless it moves the metric (2026-09-27)

- **Gates:** 5/5 (E1–E6) pass. **Verdict: chaos is REJECTED on both stacks** — the precommitted prediction held and the numbers are the deliverable.
- **Corrections / findings:** the operator's scope decision, recorded verbatim: *"the new rules dont apply to project that already exist, meaning dont redo any work, but chaos we should add to see but try it only at tje end of each stack, if it doesnt produce measurable difference reject chaos"* — so Rules 5–7 are **not** applied retroactively (nothing in BUILD-1…31 redone), canonical Rule 5's multi-position requirement is **narrowed to the end position** for this project, and the verdict rule is his. **Mechanisms, selected and rejected by DYNAMICS, verified in Python first:** logistic r = 4 (1-D unimodal sensitive dependence; 20k-step mean 0.500407 vs arcsine sd 0.3536; divergence past 0.1 by step 36) and Hénon (a, b) = (1.4, 0.3) (2-D dissipative strange attractor, folded mod 1; alive after 20k steps; divergence by step 55; distinct statistics). **The tent map is REJECTED BY MEASUREMENT before any stack test:** in f64 its binary-shift dynamics consume a mantissa bit per iteration and the orbit reaches **exactly 0.0 at step 50** (seed 0.314…) and **53** (seed 0.7) — 20k-step mean 0.001275, 99.77% of values < 0.1: a dead orbit, not a mixer. **Stack A (harness through one pad, six tasks, dissipation baseline):** fresh supports A = 4, B = 2; every arm (no-chaos / logistic / Hénon / both) carried **[4, 6, 6, 6, 6, 6]** with the **identical contamination total 16** despite mean factors 1.000 / 0.944 / 1.049 / 0.996 — **no measurable difference ⇒ rejected**, with the analytic reason on record (the residue sits orders of magnitude above the harness's absolute 1e-9 threshold, so a 0.5–1.5 scalar modulation cannot change which bins clear it — BUILD-16's consumer-scoped law again); the invariant held (first task exact; every arm's verdict fields equal the direct run's, field for field). **Stack B (action loop, 50 epochs):** chaos DID move the terminal support (81 → 83 in every chaos arm — the perturbation changes which actions are sampled) while the **frozen benefit metric was unchanged** (stabilization epoch 50 in all arms) and the **hard invariant held everywhere** (support == the closed form from each arm's OWN recorded actions) — **a merely different trajectory is not benefit** (Rule 5's own case) ⇒ **rejected**. One correction on record (mine): E1's first version asserted the attained factor extremes equal the interval's endpoints exactly; a finite orbit lives inside the interval without attaining them — corrected to containment plus an exercise check. **Nothing was promoted, so nothing was wired**: the mechanisms sit in `chaos.rs` as a candidate module with the tent rejection documented in the code, and the two additive seams the experiment needed (`ScratchPad::scale`, `ActionLoopEngine::set_pattern`) are inert when unused. L4 caught the missing log entry an EIGHTH consecutive time.
- **Lesson:** the operator's rule and Rule 5's own wording agree in the measured data — chaos at the end of a stack either does nothing (Stack A: the perturbation is swamped by the consumer's own threshold) or changes the trajectory without moving the metric (Stack B: 81 → 83 is difference, not benefit); rejecting it is a RESULT, and the rejection is now explained rather than assumed, with the mechanism available for any future stack.
- **Commits:** 96e1b5f (precommit) → the commit that adds this entry.

## BUILD-33 — the split chaos stage: the affine pieces and their orders (2026-09-27)

- **Gates:** 5/5 (F1–F6) pass. **Verdict: chaos REJECTED again — 19 of 19 arms tied at contamination 16**, zero strict improvements, zero worsenings; the precommit's prediction ("at least one order produces a measurable difference") is **FALSIFIED and kept on record** with a measured explanation in its place.
- **Corrections / findings:** D62 applied to BUILD-32's rejected whole function: the modulation was split into its **affine pieces** — gain (magnitude), rotation (phase), translation (place) — each a genuine operation (F1: gain scales magnitudes and leaves phases; rotation preserves magnitudes exactly and turns every phase; translation preserves the magnitude multiset and permutes sites; a full-lattice translate is the identity). All **3! = 6 orders** executed under **two mechanisms** (logistic r = 4; Hénon 1.4/0.3), plus the three single-piece controls each and the no-chaos baseline: **114 declared runs, 19 arms, all at 16**, invariants held (first task exact; every arm's verdict fields equal the direct run's, field for field). **Then F6 measured WHY on the real system** (the dissipated residue from task A + task B's actual contribution): every piece really changes the carried state, **gain and rotation move bin powers by up to ~2.2e1 and ~1.4e1**, **translation by ~1.7e-10 (noise)**, and **membership is unmoved in every case (6 = 6)** — the consumer's readout is **affine-blind**: the bins that move sit far above the absolute threshold (~1e-6 relative changes) and the bins near it (the residue's own) move by less than their margin. BUILD-32's gain-specific null result is thereby generalised to the whole affine group, with numbers. **THE RULE CONFLICT (surfaced, applied, not resolved):** the deployed hook (`ATOM-RULES-CONTEXT.md`, 2026-09-27 09:44) now says Rule 2 = "choose **one to three** mechanically justified candidate orders… do not default to random or exhaustive factorial sweeps", while the canonical Obsidian notes (2026-09-25/26) still say "run every unique permutation… `n!`". This build **satisfies both**: the declared set has three pieces, so the complete order space IS six executions (F5 asserts both texts are quoted in the precommit, and the notes-vs-hook discrepancy remains open for the operator). TWO CORRECTIONS, both mine: (i) F6's first version used a synthetic residue whose flat-index structure made the translation power-invariant by construction — a fixture artifact, rewritten against the real residue; (ii) F5's assertions tripped twice on line wrapping in my own document — fixed by rewrapping the quoted text, not by loosening the check. Nothing promoted ⇒ nothing wired; the mechanisms stay a candidate module with the three pieces and their declared conventions, and the two new pad seams (`rotate`, `translate`) are inert when unused. L4 caught the missing log entry a NINTH consecutive time.
- **Lesson:** splitting a function by its *dynamical pieces* (gain, rotation, translation) and permuting their order changed nothing — because the consumer's readout is a **membership decision against an absolute threshold**, and membership is blind to the entire affine group acting on the carried state; the pieces are real, the blindness is measurable, and the next chaos hypothesis is now specific: only a piece that moves the residue's **magnitude spectrum near the threshold** (a nonlinear or dissipative operation) can move this consumer.
- **Commits:** 9275431 (precommit) → the commit that adds this entry.

## BUILD-34 — with and without the base: the attribution sweep (2026-09-27)

- **Gates:** 4/4 (K1–K4) pass. The operator's directive applied verbatim (*"try experimenting with and with out and keep the stronges"*): **the strongest arm is kept per stack**, and every arm is an already-recorded, public surface — no new mechanism, no engine change.
- **Corrections / findings:** **Stack 1 (derivation composite, five arms, exact integers):** canonical **10**; carrier ablated **10** ⇒ **the epoch carrier is NOT load-bearing** (tie ⇒ keep the smaller construction); readout ablated (coherent-field feedback) **1** ⇒ **the destructive Born readout IS load-bearing** (remove it and closure collapses); base-only (single shot) **6** ⇒ **the composition above the base adds +4**; classical replacement (the authored group algorithm) **10** ⇒ for this quantity the base is a **PATH, not an information advantage**. **Stack 2 (harness composite, four arms):** canonical `scene_support` **4** vs classical replacement (intensity addition) **0** ⇒ the base is load-bearing for form/identify/judge and nothing above them — with the base removed entirely the upper stages still produce their verdicts from the task's blocks alone (closure_before 10, economy_answers 100, sufficient false → true). **Base-only measured to be not-a-verdict**: its readout is identical for two different targets (same support, same strongest off-origin bin (18,0) at power 6.400e5) while the composite's verdicts differ (false vs true) — the base creates state, the composition produces the answer. **THREE CORRECTIONS, all mine, all in the precommit's numbers:** scene_support predicted 6, measured 4; "flat = support 1" predicted, measured 0 (the readout excludes DC); the two-block closure predicted 4, measured 10 — the gate now asserts the report against the direct measurement rather than a remembered number. L4 caught the missing log entry a TENTH consecutive time.
- **Lesson:** "with and without" turns a label into a measured map — the quantum-derived base is load-bearing for STATE and for the SCENE readouts, the classical composition above it owns the behavior, and where a classical algorithm competes on the same quantity it reaches the same integer, so the base is credited as the path rather than as an advantage; the carrier ablation showing a tie is exactly the kind of ingredient this discipline is for, and the operator's keep-the-strongest rule retired it on the spot.
- **Commits:** 15e5883 (precommit) → the commit that adds this entry.

## BUILD-35 — the matched benefit test: 24 paired instances, categorical or marginal (2026-09-27)

- **Gates:** 4/4 (M1–M5) pass. Motivated by an external audit of the recorded successes whose outstanding suggestion was that "stronger statistically matched alternatives would further test quantum-specific benefit" — this build runs exactly that, on 24 paired instances rather than one, and reports categorical-vs-marginal per effect.
- **Corrections / findings:** **(B, C) CATEGORICAL** — the coherent base produces difference structure on 24/24 instances (10 bins) while the classical replacement (intensity addition) produces 0 on every one (greater 24, equal 0, lower 0): the base owns structure generation, tested 24-fold. **(E, A) CATEGORICAL TIE** — the emergent closure reaches the authored group algorithm's integer on 24/24 (10 for the base set, 50 for the ladder sets; equal 24, different 0): the quantum-derived route is an equivalent PATH, not an accuracy advantage, on this quantity — BUILD-34's single-instance finding generalized. **(E, X) MARGINAL, NOT CATEGORICAL — the precommit's prediction is FALSIFIED and the falsification is the best result of the build:** the destructive-readout ablation was predicted to lose 24/24; measured it loses on 23/24 (X = 1 against E >= 10, BUILD-2's collapse) and on **instance 6 it SATURATES at X = 200** — every conjugate pair of the 20x20 lattice, i.e. no discrimination at all. Removing the destructive readout does not merely lose the answer, it can destroy discrimination entirely; a single-instance build could not have seen this. **One generator correction (mine):** the first scene ladder wrapped an 8-point list and produced only 16 distinct scenes; M1 caught it before any claim was made, and the ladder is now 24 declared distinct points. L4 caught the missing log entry an ELEVENTH consecutive time.
- **Lesson:** a paired design over a declared instance family converts "it worked" into "it works in this direction on N of N instances, categorically, or it does not" — and the one non-categorical result (the ablation saturating on a single instance) is precisely the kind of fact that single-instance builds cannot produce; the audit was right that matched alternatives were the missing rigor, and running them made the honest claim *stronger*, not weaker.
- **Commits:** the precommit that precedes this entry → the commit that adds it.
