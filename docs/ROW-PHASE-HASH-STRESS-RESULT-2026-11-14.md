# Row-phase hashed-fingerprint stress — one-row specificity failed

Protocol frozen at `c372c0f`, unchanged two-crate source at `395ed54`,
native runner at `ae936e9`. Fresh seeds1401..1656 generated2,048
complete4,096-event trajectories (8,388,608 native steps) and
replayed every full numeric trajectory with exact bit equality.
Finite, power-bound and energy checks passed. The256 identical-input
legitimate hot-cache witnesses matched the single-fingerprint
trajectories exactly. Raw per-case CSV SHA-256:
`badcae43ae2e94961358bb377b8ad24e000c69be131e78df36271983fd853601`.
Summary `experiments/row-phase-hash-stress/RESULT.json` SHA-256:
`2824ccadde9a21067d47b8680dc026faa90e53629ebcf9ceb31a80c835e4ee49`.

| 4,096-event pattern on fresh opaque fingerprints | Phase responses /256 | Plain exact-fingerprint count /256 |
| --- | ---: | ---: |
| One fingerprint every4,000 ns | 256 | 256 |
| Two-fingerprint rotation every4,000 ns | **134** | 0 |
| Four-fingerprint rotation every4,000 ns | **42** | 0 |
| Eight-fingerprint rotation every4,000 ns | 1 | 0 |
| Sixteen-fingerprint rotation every4,000 ns | 0 | 0 |
| Sixty-four-fingerprint rotation every4,000 ns | 0 | 0 |
| 4,096 distinct fingerprints every50 ns | 0 | 0 |
| Identical legitimate one-fingerprint loop | 256 | 256 |

The two-fingerprint first crossings ranged ticks76..231; four-
fingerprint crossings ranged77..205. These distinct full fingerprints
can still have phase directions that align in the bounded complex
field. The earlier32-seed holdout included eight-row controls but no
two-/four-row controls, so its 1/96 scored-control statement did not
establish a generally selective *single-fingerprint* sensor. A
straightforward exact-fingerprint count separates these declared
patterns without this phase-alignment ambiguity. Two-/four-row
rotations are input-shape controls for this specific one-fingerprint
claim; they are **not** asserted to be physically safe DRAM behavior.

**Disposition:** retain the research source and explicit app numeric
mode, but demote its status to an experimental phase-alignment
observation. Do not call it a specific Rowhammer defense, a verified
one-row detector, or a repair of the shipped thermal path. The source,
capacity and previous results were not retuned; this stress evidence
belongs in the library manifest, GUI wording and capability map.
No physical row mapping, bit-flip evidence, attacker intent or live
protection was obtained.
