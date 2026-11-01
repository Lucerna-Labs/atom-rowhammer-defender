# Simulated thermal-boundary result — 2026-10-31

Simulation only. No CPU, PMU, or hardware ring is opened or used.

The original timestamp-only recurrence composition failed the frozen thermal
boundary: 92/132 pre-loss-labeled traces ruptured and 53/57 genuinely
subcritical controls ruptured. It could emit after a few rapid observations and
the immutable feed retained state across full cooling intervals.

The corrected composition replaces that permissive timestamp gate with a bounded
mathematical recurrence-energy state:

- repeated fingerprints contribute one heat unit;
- heat decays exponentially with a 500-microsecond time constant;
- the immutable swarm is perturbed only after heat reaches 128;
- identity memory expires and is capped at 65,536 entries;
- backward time clears transient thermal state;
- each swarm remains independent with separate state, owned copies, and receipt.

Chaos comparison on the frozen unit: plain and folded-Henon decay both pass all
visible positives and negatives, so plain wins the tie. Logistic contribution
and combined arms rupture three boundary controls and are rejected.

Integrated results:

- delivered-stream thermal gate: 33/33 oracle-positive traces rupture;
- loss/noise transformations below the delivered danger boundary: 0/99 rupture;
- independent subcritical/cooling controls: 0/57 rupture;
- exact second replay byte-identical, SHA-256
  `7c4a45d0ec838e18e99e456d71851a7d41a6733be184e6d942fd36b9f195f03c`;
- broader timed adversarial corpus remains 2,048/2,048 attacks and 0/123
  controls, SHA-256
  `d351cd03abe5f0357e7f2c36aef7746a8100d6fbc591298fe70184f4311af987`;
- untimed known-technique corpus remains 8,704/8,704 attacks and 0/2,048
  controls, SHA-256
  `b5dc4e3fb4be91401f0b7c92394aef1dbeb8c229ebeee65fea63e1f723f12677`;
- full Rust workspace: 76 passed, two existing privileged hardware tests ignored.

The 99 transformed negatives retain their pre-loss labels as an explicit hidden-
information stress result. Claiming they must rupture would require an observation
source outside the authorized simulator boundary. This result establishes 100
percent response and zero false rupture on the declared delivered-observation
simulation corpora, not universal or physical Rowhammer effectiveness.
