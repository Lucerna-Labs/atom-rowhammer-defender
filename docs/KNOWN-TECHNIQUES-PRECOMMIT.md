# Known-technique black-box stress corpus — precommit, 2026-10-31

External evaluator only. Runtime receives numeric addresses and no technique name,
label, expected output, or policy.

Generate address-order representations of single-sided, double-sided, one-location,
n-sided4/8/16/32/59 (TRRespass class), Blacksmith nonuniform frequency/phase/
amplitude, Half-Double far+bridge, RowPress repeated-row proxy, SledgeHammer
parallel-bank interleave, ZenHammer refresh-aligned burst proxy, Phoenix128 and
Phoenix2608 sparse refresh-interval proxies, eviction-based hammering, and remote
burst delivery. Run8 translations x16 seeds x4 observation-loss levels per family,
1024 observations before loss, with deterministic noise variants. Controls:
uniform random, sequential, nonrepeating stride and random permutation.

Required for the frozen modeled corpus: every externally labeled attack trace
produces at least one emergent rupture, every control stays intact, exact replay on
a held-out witness subset, no panic, and app source/binary unchanged by evaluation.

RowPress/Zen/Phoenix/Sledge/remote cases are address-order proxies only: the current
input lacks actual ACT/PRE/REF, bank and row-open timing. Report them separately and
do not claim physical technique coverage from proxy success.
