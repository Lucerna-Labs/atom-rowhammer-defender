# Simulation observation-boundary correction — 2026-10-31

Jesse rejected CPU-ring acquisition. The app's authorized test boundary is now
the complete generated numeric observation stream supplied by the simulator.
Events deliberately removed before that boundary are not runtime observations.

The first thermal-boundary evaluator incorrectly kept the attack label from the
pre-loss hidden trace even when the delivered trace no longer crossed the frozen
mathematical danger boundary. This produced an impossible requirement: the same
short delivered sequence could represent either a subcritical simulation or an
attack whose decisive events were withheld from the app. No label-free runtime
can distinguish identical observations by an unobserved history.

No case is deleted. Results are split explicitly:

- the delivered-stream thermal oracle is the effectiveness gate;
- the pre-loss label is retained separately as an information-loss stress result
  and may not be claimed as detectable without an authorized observation source.

On the frozen 132 transformed traces, 33 delivered streams remain
oracle-positive and 99 become oracle-negative. The plain decaying recurrence
candidate ruptures 33/33 visible positives, 0/99 visible negatives, and 0/57
independent controls. It therefore corrects both observed false positives and
visible misses without labels or CPU rings.

Chaos construction comparison at the same frozen threshold:

- plain: 33/33 visible positives, 0/156 negatives;
- folded-Henon decay perturbation: ties plain, rejected because plain wins ties;
- logistic contribution perturbation: same positive coverage but ruptures three
  boundary controls, rejected;
- combined: same three control regressions, rejected.

The evaluator, frozen model, pre-loss results, and information limit remain in
the evidence record. This correction changes which events are inside the
authorized simulation boundary; it does not tune the danger threshold after the
result.
