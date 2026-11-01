# Immutable secondary swarm experiment — precommit, 2026-10-31

Black-box evaluation found cycle8, cycle16 and burst8 traces never reach the
immutable stack because the conventional concentration window emits zero. The
Atom stack itself contains no family labels or classifier.

Test additional independent instances of the unchanged immutable swarm under
fixed content-independent projections. Every instance has its own owned copies,
state and receipt. No voting or cross-stack confirmation. External evaluator
labels remain outside runtime.

Candidate channels: direct `fingerprint mod8`, paired `floor(fingerprint/2) mod8`,
and quartet `floor(fingerprint/4) mod8`, each receiving every observation. Compare
each alone and their any-rupture union against the frozen5120-trace corpus.
Required adoption: all4096 external modeled-attack traces rupture and all1024
uniform/sequential controls remain silent, exact replay. If direct projections
rupture controls, preserve failure and do not integrate.
