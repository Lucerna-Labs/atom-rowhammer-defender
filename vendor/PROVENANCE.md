# Vendored crates provenance

- `vendor/nocopy/*` — byte-copies of the no-copy atom crates from
  `/home/jesse/Desktop/Projects/imperium-main/imperium-main/encrypted key protection/`
  (workspace root manifest and Cargo.lock removed so this repo's root owns
  workspace inheritance; `nocopy-demo` dropped as unneeded).
- `crates/imperium-nocopy-vpn-observer`, `crates/imperium-nocopy-swarm`,
  `crates/imperium-nocopy-swarm-live` — byte-copies from
  `imperium-main/crates/`; only edits: path deps rewritten from
  `../../encrypted key protection/X` to `../../vendor/nocopy/X`, and the
  external dev-dependencies of `imperium-nocopy-vpn-observer` (imperium-net-mesh
  etc.) stripped — its test module only uses vendored crates.
