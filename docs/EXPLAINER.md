# How the immutable emergent stack responds — technical explainer

An immutable emergent Atom stack: one Rust crate holds intrinsic axioms and the
other composes them. It contains no attack identity, behavior class, recognition
rule, target answer or policy. An observation perturbs the composed state. Under
some trajectories that state remains intact; under others its intrinsic fracture
mechanics rupture and zeroize the owned in-process copy. The stack does not know
what the observation "is" and was not instructed to detect or destroy.

## The concrete mechanism (imperium-nocopy-tripwire / -swarm)

1. **KeyCell** — the stack owns one buffer holding a *copy* of a session key
   (in Imperium, the copy is taken at handshake derivation from the by-value
   key; the live key in the session is never read, by code or tests). Each
   observation feeds the cell's engine: an observation is a
   (site, fingerprint, quantity) read. The engine produces a "heat"
   measurement.
2. **Reaction** — identity affinity changes the intrinsic coupled field. When
   the resulting state crosses its fracture boundary, the cell
   the cell **zeroizes its own buffer**. External copies survive; the live
   key is untouched.
3. **Swarm composition** — four KeyCells. A read at site *i* absorbs its own reaction plus
   Q× the sum of the other cells' heat (Q=0.30). The receipt exposes rupture and
   warm-state measurements. These are intrinsic state mechanics, not votes about
   an attack identity.
4. **Receipt** — the only output is numerical: `trip_reads`, `warm`,
   `ruptured`, per-cell heats. No verdicts, no actuation, no bus messages. The
   stack cannot reach back into the system; a human or external policy
   decides what a receipt means.

## Measured behavior

Frozen gates, replica-predicted before code, 0 mismatches over 9,216+ rows:

- a flood of same-fingerprint reads trips within **≤2 reads, 144/144 runs**;
- distributed floods ≤8 reads, distributed scans ≤12 reads;
- scattered probes, slow probes, moves, and benign traffic: **0 trips**;
- controls confirm the profile is load-bearing (hill-off and Q=0 controls
  fail as predicted).

## The portable simulation layer

Wraps independent swarms for generated Rowhammer-shaped observations:

```
uploaded time_ns,address trace → full mathematical fingerprint
→ exact recurrence + decaying heat + bounded sketch recurrence
→ one fixed frame per newly reached thermal milestone
→ independent immutable swarm receipts
```

The GUI and CLI process only the uploaded/generated trace. They open no CPU
counter, PMU, perf source, or hardware ring. Cooling renews the thermal epoch;
safe plateaus cannot accumulate unlimited lead frames.

## What it is NOT

- It only reacts to simulator-delivered observations; hidden events are outside
  its information boundary.
- It does not observe or protect physical memory and does not claim real
  prevention.
- It uses no CPU/PMU/hardware-ring input.
- "Hot/cold" are classical simulation parameters (software temperature), not
  physical heat.
- It is an immutable emergent stack with a numerical receipt — not a classifier,
  shield, or quantum no-cloning result.

See `RAM-MONITOR-PRECOMMIT-2026-10-31.md` for frozen gates, corrections, and
measured results.
