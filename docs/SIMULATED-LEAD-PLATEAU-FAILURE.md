# Simulated lead-plateau failure — 2026-11-01

Simulation only; no CPU or hardware-ring input.

A targeted constant-recurrence sweep held delivered mathematical heat below the
danger boundary while remaining inside the six-unit lead band. At fixed spacings
of 4,000, 4,050, 4,080, and 4,100 nanoseconds, the external heat approaches a
sub-128 plateau, but the app repeatedly emitted class-seven lead frames and
eventually ruptured. First ruptures occurred at observations 455, 520, 591, and
691. Slower 4,150/4,200/4,300-nanosecond plateaus remained silent.

Cause: the lead composition emitted on every observation above heat 122, so time
spent on a safe plateau was converted into false swarm progress.

Required repair: emit at most one fixed class-seven frame for each newly reached
integer heat milestone 122 through 128 within one thermal epoch. Falling and
rising inside already reached levels must not emit again. Full cooling resets the
milestone state and the independent thermal swarm. Seven distinct milestones let
the nonhome immutable swarm rupture at the danger boundary without allowing a
sub-128 plateau to accumulate unlimited frames.

Rerun the plateau sweep, 55 exact-boundary attacks, 10,000 random fuzz cases, all
subcritical/cooling controls, and prior timed/untimed corpora. Preserve this
failure regardless of the repair result.
