# No-CPU portable trace input — precommit, 2026-11-01

Jesse explicitly did not authorize CPU-ring acquisition. The current GUI still
defaults to the legacy perf source and exposes per-CPU/PEBS controls, so merely
reverting the later hybrid bridge did not fully correct the app.

Remove CPU/PMU/perf-ring acquisition from this branch and from the GUI/CLI
surface. Preserve historical evidence in Git, but delete the active perf module,
live perf tests, perf settings, and libc dependency from the simulation app.

Replace the source boundary with a portable text trace:

```
# time_ns,address
0,0x1000
50,0x1040
```

Requirements:

- parser uses only std, accepts decimal or `0x` integers, monotonic timestamps,
  comments and blank lines;
- malformed, backward-time, missing-field, empty, or oversized traces fail
  closed with a line-specific error;
- CLI accepts an explicit trace-file path and returns numerical receipts;
- GUI starts with no source and accepts a dropped trace file, showing the file
  name/error and processing only that uploaded content;
- no default external connection, CPU counter, PMU, ring, privilege, or hidden
  background capture;
- AI callers can invoke the same CLI trace-file boundary;
- existing synthetic replay, thermal, fuzz, capacity, and workspace gates remain
  green.

No external dependency may be added. This is a simulation input correction, not
physical monitoring.
