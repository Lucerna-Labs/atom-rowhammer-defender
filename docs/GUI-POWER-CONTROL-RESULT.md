# GUI ON/OFF control result — 2026-11-01

The portable GUI now has a large explicit **TURN ON / TURN OFF** control.

- normal startup is OFF;
- loading or dropping a trace only stages it and leaves sample count at zero;
- ON starts the staged trace from its beginning with fresh stack state;
- OFF clears the active source and stops sample growth;
- ON after OFF restarts from the beginning;
- Apply and Reset return the simulation to OFF;
- HALTED remains distinct from operator OFF.

Controller gates verify staged-upload silence, ON execution, immediate OFF sample
stability, restart behavior, reset behavior, and stricter-settings behavior.
Full workspace: 79 passed, zero ignored. Release GUI remained alive for the
five-second launch check.

Candidate hashes:

- CLI `3c8ad94ae3d4ca97c2eedf089d2e439a3970a4547767f69227975843f583e347`;
- GUI `9d5e7f717a64aa7c2d60940bc2d371f37c851d2a586dd0098b68fdadd0fb2f75`.

The obsolete CPU-ring systemd unit was backed up, removed from `/etc/systemd/system`,
and daemon-reloaded. `systemctl status ram-monitor.service` now returns unit not
found; no background monitor process exists. Recoverable backup SHA-256
`0e29a4ef0c65ea66b61b6cd7ad26fce17888740f3592e06d7c825745efe854a2`
is under `outputs/removed-cpu-ring-service-v1`.
