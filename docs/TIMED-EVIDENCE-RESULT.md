# Timed model and address-evidence result — 2026-10-31

This is an isolated, uninstalled candidate. The running service remains v3,
PID1238418, installed CLI SHA256 a13e1ac253c7ad9fdee2fe999804df315bd867533bb9c030e6f8428c7f703f34.

## Corrected simulation

The original160-case address-only evaluation could not establish physical
Rowhammer danger. The new72-case model tracks a synthetic bank's open row,
cache hits, explicit precharge/reopen, time, refresh epochs and neighboring
row disturbance. Each ACT adds one unit to each adjacent victim; refresh
resets charge. The32/128-unit budgets,1/64ms refresh periods,8192-byte row
mapping and10us/10ms step sizes are illustrative inputs, not measured host
DRAM parameters. A separate grouped activation-accounting oracle matched all
72 stateful model outcomes. All candidate outputs replayed exactly.

Of72 modeled cases,10 reached the model's disturbance budget. The candidate
warned before the modeled fault in6 and missed4 distributed8-/16-row cases.
It alerted on26 of62 cases without a modeled fault. These selected profile
counts are not real-world accuracy estimates. Fast/slow versions and open-row
versus forced-reopen behavior can share address patterns despite different
physical outcomes. Thus the old labels were inadequate, and the current
concentration gate is still not an activation-risk detector.

## Staged code correction

The candidate retains full shifted addresses in the concentration window,
scopes virtual/instruction identities by PID and source, and preserves kernel
PERF_SAMPLE_TIME. It keeps the vendored Atom swarm unchanged; only the class
projection uses the legacy mask. CLI/GUI expose evidence provenance and state
that physical row activation is unmeasured. The repeated16KiB-stride alias
controls from the original fixture dropped from8/8 alerts to0/8. The other
original fixture outcomes stayed unchanged. All160 full-address frequency
checks passed. Workspace tests, PID/source separation tests, wrapped payload
and timestamp parser tests passed.

In the timed model some old distributed-pattern alerts came from address
collisions, so removing those collisions exposes missed dangerous model
cases. This blocks deployment as a comprehensive detection improvement.
Timestamps are now carried; they are not yet used by a calibrated time/rate
risk mechanism. Per-CPU sample streams also require an explicit ordering rule
before a global temporal decision. No threshold was tuned to hide these misses.

## Hardware capability correction

Live inspection reports **no detected virtualization** and Intel i7-13700K.
The previous KVM/no-data-address assertion was not verified on this host and
is withdrawn. The production system-wide path explicitly disables precise
address events even with --privileged; its fallback proves only that the
selected event lacks data, not that the hardware does.

A separate unprivileged C probe used the sysfs mem-loads events and a bounded
ordinary random-memory workload, without cache flushing or physical hammering.
On CPU0 (cpu_core) it received650 samples, all650 with nonzero data addresses
and timestamps, lost0. On CPU16 (cpu_atom) it received2378 samples, all2378
with data addresses and timestamps, lost0. P-core probing used the documented
auxiliary event group. Only aggregate counts were saved. These are virtual
data addresses from precise load events; physical address mapping, DRAM ACT
commands and host vulnerability thresholds remain unverified.

The next implementation boundary is selecting the appropriate hybrid PMU
precise memory events in production and evaluating time/locality against
independently modeled danger and real ordinary workloads. It must account for
the changed event semantics and sampling overhead; the diagnostic event is
not automatically equivalent to the existing cache-miss stream. No new live
service installation or Secrets MCP privileged action occurred in this task.

Sources: [ANVIL](https://www.seclab.cs.sunysb.edu/seclab/pubs/ANVIL.pdf),
[Linux perf memory-event groups](https://github.com/torvalds/linux/blob/master/tools/perf/arch/x86/util/mem-events.c),
local `/usr/include/linux/perf_event.h` and PMU sysfs descriptors. Artifacts:
`timed-results.json`, `identity-results.json`, `pmu-results.json`, probe sources
and `workspace-tests.log`.
