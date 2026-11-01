# Immutable secondary swarm discovery result — 2026-10-31

External black-box corpus:5,120 traces,4,096 modeled attacks,1,024 uniform/
sequential controls,512 samples each,8 translations,64 seeds. Labels existed only
in the evaluator. Every runtime component received only numeric observations.

Round1 direct fixed projections (`fp mod8`, `floor(fp/2) mod8`,
`floor(fp/4) mod8`) ruptured every control and were rejected.

Accepted fixed observation compositions, each feeding its own unchanged immutable
swarm instance:

1. recurrence observer A:64-sample window, emit the current full fingerprint only
   after its third prior recurrence (count>=5). It ruptures single, alternate2,
   cycle4, cycle8, burst8, diluted25 and mixed75; cycle16 stays silent; uniform and
   sequential controls stay silent.
2. recurrence observer B: fixed `floor(fingerprint/2)` projection,64-sample window,
   emit only at count>=7. It ruptures all eight attack families including cycle16;
   uniform and sequential controls stay silent.

The any-local-rupture union covers4096/4096 modeled attacks with0/1024 control
ruptures. No stack identifies a family, and neither stack reads or confirms the
other. The conventional observers are fixed and content-independent; the swarms
remain immutable.

Both immutable secondary swarms are now integrated inside `RamMonitor`. The full
5120-trace black-box corpus was rerun against the integrated source:4096/4096
modeled attacks ruptured,0/1024 controls ruptured. Permanent regression gates
cover cycle8, cycle16, burst8, controls, and operator disable. App tests pass.
