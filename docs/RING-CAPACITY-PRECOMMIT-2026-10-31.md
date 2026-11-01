# Live perf capacity candidate — precommit before code

The previous repaired system-wide service used 8 data pages per CPU, read 24
CPU rings in one thread, then slept 50 ms. At one LLC miss per sample, its
corrected parser observed millions of real lost samples; per-loop logging
amplified the load. That build was rolled back. The later log-throttled
candidate `368ac59` is staged but has not run system-wide. Linux kernel perf
ring documentation says a non-overwrite ring emits lost records when its
consumer cannot keep up. This host reports `perf_event_mlock_kb=516`, so a
64-page (256 KiB) data ring per CPU is within that per-CPU allowance.
Sources: https://docs.kernel.org/userspace-api/perf_ring_buffer.html and
https://docs.kernel.org/5.10/admin-guide/perf-security.html.

## One candidate, no detection weakening

Keep `sample_period=1` and the same detector, fingerprint geometry and
thresholds. Increase each ring from 8 to 64 data pages and poll the 24 rings
every 5 ms instead of 50 ms. Keep heartbeat/gap reporting tied to elapsed
wall time (about 60 s), not loop count. The larger ring tolerates a longer
burst; faster draining reduces how long the kernel waits for free space.
There is no claim that either guarantees zero loss under every workload.

## Frozen gates

1. UAPI lost-record and wrapped-ring fixtures, workspace tests, CLI validation,
   and the three staged Bug Finder suites must remain green. The default
   hammer/scatter/mixed/benign replay outputs and valid CLI behavior must stay
   unchanged. An ordinary user self-perf open with the 64-page ring must
   succeed on this host; this does not stand in for privileged all-CPU mode.
2. A live system-wide trial requires a new exact Secrets MCP approval for
   installing the hash-pinned candidate executable. Existing approved stop,
   start and old-binary restore commands provide rollback. The installed
   candidate must match its staged SHA-256, run under a new stable PID, and
   report the corrected lost count and address fallback.
3. Under the then-current host workload, two successive 60-second heartbeat
   intervals should have **zero new lost samples** and the daemon should
   remain active, with no more than one gap line per interval. If loss grows,
   service restarts, or source checks fail, the live gate is FAILED and the
   exact old-binary rollback commands run. A zero-loss observation is bounded
   to those two intervals, not proof of completeness under all traffic.
4. Maintain the existing no-physical-address limitation: positive
   `ip_fallback` prevents interpreting a trip as a confirmed physical row.
   No actual Rowhammer traffic will be generated for this capacity trial.

Prediction: synthetic/replay gates MEASURED. Live opening with 64 pages is
expected to work based on the host mlock limit. Loss reduction is an empirical
question; do not mark the live gate MEASURED until the two windows pass.
