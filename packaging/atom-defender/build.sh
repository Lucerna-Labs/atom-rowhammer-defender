#!/bin/sh
set -eu
repo=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$repo"
[ "$(dpkg --print-architecture)" = amd64 ]
sha256sum -c crates/atom-keyboard-guard/kernel/SHA256SUMS
(cd vendor/network-lightcone-exact && sha256sum --quiet -c SOURCE.sha256)
dest="$repo/target/atom-defender-package"
mkdir -p "$dest"
stage=$(mktemp -d "$dest/stage.XXXXXX")
trap 'rm -rf -- "$stage"' EXIT
export CARGO_TARGET_DIR="$repo/target/pressure"
cargo build --locked --offline --release -p ram-monitor -p ram-monitor-gui -p atom-credential-guard -p atom-keyboard-guard -p atom-network-lightcone
install -d "$stage/DEBIAN" "$stage/opt/ram-monitor" "$stage/usr/share/applications"
for bin in ram-monitor ram-monitor-gui atom-credential-guard atom-keyboard-guard atom-network-lightcone; do
    install -m 0755 "$CARGO_TARGET_DIR/release/$bin" "$stage/opt/ram-monitor/$bin"
done
install -m 0644 packaging/atom-defender/atom-rowhammer-defender.desktop "$stage/usr/share/applications/"
install -d "$stage/usr/share/doc/atom-defender"
install -m 0644 LICENSE COMMERCIAL.md packaging/atom-defender/copyright crates/atom-keyboard-guard/kernel/keyboard.bpf.c "$stage/usr/share/doc/atom-defender/"
install -m 0644 vendor/network-lightcone-exact/IMPERIUM-LICENSE vendor/network-lightcone-exact/PROVENANCE.md "$stage/usr/share/doc/atom-defender/"
for component in integrity-e013 cognitive-engine-upstream spiderweb-bus-upstream network-lightcone-exact; do
    install -m 0644 "vendor/$component/LICENSE" "$stage/usr/share/doc/atom-defender/LICENSE-$component"
done
install -m 0755 packaging/atom-defender/postinst "$stage/DEBIAN/postinst"
cat > "$stage/DEBIAN/control" <<'META'
Package: atom-defender
Version: 0.1.2+20261124
Architecture: amd64
Maintainer: Lucerna Labs
Depends: pkexec, libpcap0.8t64 | libpcap0.8, libbpf1, libcap2-bin, libc6, libgcc-s1
Section: utils
Priority: optional
Description: Atom Defender with selected credential file access control, Network Lightcone and keyboard access monitoring
META
pkg="$dest/atom-defender_0.1.2+20261124_amd64.deb"
dpkg-deb --root-owner-group --build "$stage" "$pkg"
sha256sum "$pkg"
