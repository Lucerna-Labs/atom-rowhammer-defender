# Memory-stall dwell material — bounded numerical result

The first protocol was frozen at `6a60dba`; the two owning Rust crates
were frozen at `9069fb3`; the evaluator at `17db623`. The separate
block-length stress protocol was frozen at `c35c990`. The native engine
accepts a unit scalar per tick and returns state, without event labels,
process identity, attack routing, policy, I/O or mitigation. The
candidate ran on a classical CPU using quantum-derived density mathematics.

The main matrix has1,846 trajectories ×96 ticks =177,216 native ticks.
All rows passed finite, trace1, PSD, capacity and unit-state checks.
Seventy-one A/full/plain traces repeated byte-identically; all64 heldout
ones matched debug and release byte for byte. Raw tick CSV is compressed
as `experiments/memory-stall/raw.csv.gz`, SHA-256
`ae182c03aada7e2456906b433744861157672ba5f5d821c85362eca62beaac0e`.
Trajectory summary `matrix.csv` has SHA-256
`4bef9c7d8dd635b4824831277145c21bbd3d37d962505db492cfcfa7d7ad97bb`.
The full locked offline Rust workspace suite and portable-only guard passed.
The three plain CSV artifacts were normalized from the Python CSV writer's
CRLF line endings to LF after generation; numeric rows were unchanged,
and the hashes here and in JSON receipts are for the committed LF copies.

| Arm | Heldout 32-tick dwell responses /32 | Equal-histogram isolated pulse responses /32 |
| --- | ---: | ---: |
| A/full/plain | 32 | 0 |
| B/full/plain | 32 | 0 |
| Power-matched diagonal, A | 0 | 0 |
| Matched classical population diffusion, A | 0 | 0 |
| A/no chemistry | 0 | 0 |
| A/no mechanics | 0 | 0 |

All32 paired dwell mean energies exceeded their own pulse controls.
Mean peak energy was0.0727965 for full dwell,0.0005241 for full pulses,
and0.000003156 for no-quantum dwell. The quantum-derived coherent base is
load-bearing relative to the two declared population controls for this
function. The no-quantum and diffusion controls are mathematically near
identical at this two-state input because both remove inter-step
coherence; they are not independent evidence of a broader quantum edge.
No-information, no-cognition and no-physics changed all32 continuous
mean-energy trajectories, but kept the same crossing counts. Removing
chemistry or mechanics zeroed the material readout by its structural
product law. The construction quiet and eight-tick burst stayed below
capacity. The benign sustained workload witness was byte-identical to
the high dwell and crossed at the same tick53, establishing the intent
identifiability limit at this input boundary.

All five chaos arms and B order tied the full plain scored response
counts on the frozen matrix, so A/full/plain/seed0 remains selected.
Some individual perturbation steps survived their intrinsic free-energy
selector; that did not produce strict task benefit. No chaos arm was
promoted and no thresholds were changed.

The independently frozen 192-profile block-length audit (1,344 native
trajectories) had these A/full/plain responses from32 equal-histogram
profiles per length: length1 0, length2 0, length4 0, length8 0,
length16 18, length32 32. `stress.csv` SHA-256 is
`275239279a6b43c9a7cbae703c35bc5bfb4b108c5171421849f50049be8c6e06`.
A plain external 16-sample rolling mean and 16-consecutive-high rule
each detected all32 length16 and all32 length32 profiles and none
at shorter lengths. Thus ordinary rules match or exceed this bounded
task; no exclusive quantum algorithmic capability or speedup is claimed.
Some legitimate 16-tick scheduled workloads cross the Atom capacity,
and sustained legitimate pressure is indistinguishable from hostile
pressure when the scalar trace is identical.

**Admitted scope:** an intrinsic numeric sense of 32-tick high memory
stall dwell versus equal-count isolated pulses on the frozen synthetic
range (.78–.90 high fraction). It met the precommitted bounded gates.
It has no live collection or app bridge yet, no attacker attribution,
no resource remediation, and no evidence for realistic workload false
alarm rates. The explicit classical comparator is at least as capable
on this corpus. Any app integration must preserve these limits and
report numerical state only.
