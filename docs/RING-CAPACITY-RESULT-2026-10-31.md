# Atom Rowhammer Defender live capacity result — 2026-10-31

Precommit: `RING-CAPACITY-PRECOMMIT-2026-10-31.md` at `157484a`.
Source implementation: isolated `codex/rowhammer-repair` commit `a98f7cb`.
This source branch incorporates a byte-for-byte snapshot of the main checkout's
previously uncommitted self-filter changes, plus the deterministic tie, CLI,
lost-record, ring-wrap and log-throttle fixes from earlier commits.

## Measured gates

| Gate | Status | Evidence |
| --- | --- | --- |
| Workspace and UAPI/replay controls | MEASURED | `cargo test --offline --workspace` passed; wrapped header/body and true lost-count fixtures passed; user-scope 64-page perf open returned 61 samples and zero lost. |
| Staged process boundary | MEASURED | Three Atom Code Bug Finder Harness suites against the staged release CLI: valid replay 200 calls/arm, repeatability 72, invalid CLI 80; all controls and source/config/harness hash gates passed; zero violations. Witness replay UNRUN, chaos benefit FAILED on ties. |
| Live system-wide launch | MEASURED | Exact approved install operations placed CLI SHA-256 `a13e1ac253c7ad9fdee2fe999804df315bd867533bb9c030e6f8428c7f703f34` and GUI SHA-256 `27ff52ff9c3e4233d0c75139ae77f901bb84d4d62844bf213bda20b22b56aad3` in `/opt/ram-monitor`; service became active with stable PID 1238418, invocation `3b9230942edd4a61bf4a5e90f1e94214`, and opened 24 CPU counters. |
| Two loss-free heartbeat intervals | MEASURED | At 18:17:53 EDT: frames 910003, trips 1, lost 0, IP fallback 2612995. At 18:18:53: frames 1004888, trips 1, lost 0, IP fallback 4043192. A third at 18:19:53: frames 1123352, trips 1, lost 0, IP fallback 5277725. Same PID/invocation; no `coverage_gap` line. This is bounded to these host workloads and intervals. |
| Post-fix live Bug Finder | MEASURED | 24 read-only calls/arm, no violations; off/on first-hit scores 27/27; controls, hashes and budgets passed; replay UNRUN and chaos benefit FAILED. Receipt in `outputs/rowhammer-repair-v3/live-after-results.json`. |
| Physical DRAM row attribution | UNRUN | The virtual PMU supplied many instruction-address fallbacks; this test did not observe real data/physical row addresses or generate Rowhammer traffic. The app remains observe-only. |

The GUI release binary stayed running during a five-second demo-mode smoke
check (terminated by the test timeout). The two temporary v3 install approvals
were removed after acceptance. The four exact old-binary rollback approvals
remain in Secrets MCP at Jesse's request; its sudo-password entry is
`agent=off, run=off`, and no raw password was exposed. The root-owned prior
binaries remain under `/opt/ram-monitor/backups/bugfinder-repair-20261031T174501/`.

## Scope and history

The first repaired live build (`2a329d0…`) failed due to rapid actual sample
loss and per-loop logging amplification. It was rolled back using the
approved store, and its failed result remains on record. This candidate kept
one LLC miss per sample, increased each ring from 8 to 64 pages, and drained
every 5 ms rather than 50 ms. It tied heartbeat reporting to elapsed time.
The run shows no loss in three observed one-minute intervals; it does not
prove no loss under heavier traffic or certify Rowhammer prevention.

Linux perf ring behavior and mlock allowance were checked against the
[kernel ring documentation](https://docs.kernel.org/userspace-api/perf_ring_buffer.html)
and [perf security documentation](https://docs.kernel.org/5.10/admin-guide/perf-security.html).
