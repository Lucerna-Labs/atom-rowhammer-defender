# Exact-path Network Lightcone

Production code comes from the local Atom Stack Store deposit
`imperium-door-one-path-lightcone-service/source/imperium-main/imperium-main`.
Its manifest records the Imperium checkout at `860a5d089370f625700d6fd588ba72d78b40a29b`
with local edits. The preserved production files include the no-fold PathWorld loader
and its hardened PATHPACK decoder. SOURCE.sha256 pins the actual bytes of every
imported crate manifest, source file, provenance notice and embedded role pack.
Check it from this directory with `sha256sum -c SOURCE.sha256`.

The three embedded role packs are synthetic documentation-network examples. No captured network training paths are included. Two test modules use the synthetic path count. The desktop selects its own reference pack at runtime. Its Spiderweb dependencies resolve
to Atom Defender's existing pinned Spiderweb implementation. The worker uses the
original LightconeService, LightconeStrand, allow/deny PathWorld and receipt ledger.
The external host reference pack is selected at run time and is never packaged.

The harvested workspace declares MIT in its Cargo metadata. The public Imperium
repository also carries the separate license text included as IMPERIUM-LICENSE,
Git blob `2a47a8bcdef071ef01f8d3ed199b1ba1275cc446`.

The rights holder offers the bundled Imperium portions under the accompanying MIT LICENSE. IMPERIUM-LICENSE retains the original upstream notice.
