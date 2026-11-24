# Row-fingerprint phase memory — bounded synthetic companion

> **Correction after independent hash stress at `967e5ca`:** this
> first 32-seed gate passed only its declared slow/unique/eight-row
> controls. Fresh full-fingerprint sets caused responses on134/256
> two-row and42/256 four-row rotations. The source is retained as an
> experimental phase-alignment observation, **not** admitted as a
> selective one-fingerprint or physical Rowhammer defense. See
> `ROW-PHASE-HASH-STRESS-RESULT-2026-11-14.md`. The measurements below
> remain unchanged as the historical bounded result.

Protocol `ROWHAMMER-PHASE-MEMORY-PRECOMMIT-2026-11-14.md` was frozen
at `660335e`, with pre-execution corrections at `a58f496`,
`cc88c74`, `de134d3` and `10ff478`. Native two-crate source was frozen
at `395ed54`; external evaluator at `05ff693`. The engine sees only
`(time_ns,fingerprint)` and returns complex field/material state. It
receives no source name, attack label, physical row map or action.

Both construction orders and all five chaos variants tied plain on
the frozen scored construction cases (score1); A/full/plain/seed0
remained selected. Construction B changed continuous peak values but
not crossings. The 32-seed heldout comparison used the same uploaded
synthetic addresses/timestamps for the new engine, unchanged shipped
monitor and ordinary exact-fingerprint count. All238 new plain
trajectories repeated byte-identically, all224 heldout plain outputs
matched debug/release byte for byte, and finite/complex-power/
bounded-material/time checks passed. Full locked offline workspace
tests and portable-only guard passed. Matrix
`experiments/row-phase/matrix.csv` SHA-256:
`c93280ccb69bca18cc7fe466158fa02049f373d4769298c1a98b92a3b408e8be`.
Selected complete143,360-tick raw numeric rows are in
`selected-raw.csv.gz`, SHA-256
`5acfa5920b87280cd4f4eff5023c6d066131b4a2b2d68458d76d0b0315258ae1`;
all other trajectory stdout hashes and summaries are retained and
reproducible from the frozen generator/source.

| Arm, fresh32-seed traces | Same row ~4 µs /32 | Same row ~5 µs /32 | Slow same row /32 | Unique fast rows /32 | Eight-row rotation /32 | 64-row fast rotation /32 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| New complete plain | 32 | 32 | 0 | 0 | 1 | 0 |
| New no-quantum or diffusion | 0 | 0 | 0 | 0 | 0 | 0 |
| Unchanged shipped Rowhammer monitor | 6 | 0 | 0 | 0 | 6 | 32 |
| External OR of those two numerical observations | 32 | 32 | 0 | 0 | 7 | 32 |
| Ordinary >=100 exact-fingerprint hits in500 µs | 32 | 26 | 0 | 0 | 0 | 0 |

The full stack first crossed on the ~4 µs heldout targets at ticks75–76;
peak material mean was0.02830. It rescued26 shipped misses and added
one response to the shipped scored controls, meeting the precommitted
bounded companion gate. The one eight-row false response was seed1326
at3,960 ns spacing, first crossing170 with peak0.01096; its opaque
hashed phases aligned enough to accumulate. The shipped monitor's
32/32 fast64-row rotation responses are an existing unscored limitation,
not a new success or a control solved by this stack. No-chemistry and
no-mechanics removed all new target responses. Removing information,
cognition or physics changed continuous peak material, but did not
remove target crossings. All chaos variants tied construction and
were not promoted.

The new field's coherent phase accumulation is load-bearing relative
to its declared phase-free population arms. A conventional complex
accumulator can implement the same phase math, and the ordinary
exact-fingerprint counter was stronger on the scored controls; no
quantum-computing speedup or exclusive algorithmic capability is
claimed. A byte-identical legitimate hot-cache-loop witness caused
32/32 new responses, so the input cannot infer hostile Rowhammer
intent. Fingerprints are mathematical 64-byte blocks from uploaded
addresses, not verified physical DRAM rows.

**Admitted scope:** one additional bounded synthetic fast-repeat
numeric function in its two unchanged owning Rust crates. It does not
repair the shipped thermal gate, predict bit flips, read live hardware,
or prevent damage. Its app boundary is not yet wired; any bridge must
remain independent of the old Rowhammer control path and report
numerical state without an attacker verdict.
