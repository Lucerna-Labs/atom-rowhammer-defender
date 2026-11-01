# Simulated recurrence-capacity edge — precommit, 2026-11-01

Simulation only; no CPU or hardware-ring input.

The thermal observer hard-caps exact fingerprint memory at 65,536 entries. Test
sustained delivered recurrence at periods 512, 1,024, 2,048, 4,096, 8,192,
16,384, 32,768, 65,535, 65,536, and 131,072 under 10/25/50-nanosecond spacing.
Each trace stops at the first external danger plus the minimum response tail.

Matched controls are equally long fast streams whose fingerprints never repeat.
Labels remain external. Required for each capacity admitted by the implementation:
oracle-positive recurrence ruptures and unique controls remain intact. Test the
exact cap and one larger period explicitly rather than inferring behavior.

Any cap-induced miss is preserved before adding an independent edge composition.
Do not remove the hard memory bound, add labels, or use CPU rings. This is a
synthetic state-capacity gate, not a claim that every period is physically useful
for Rowhammer.
