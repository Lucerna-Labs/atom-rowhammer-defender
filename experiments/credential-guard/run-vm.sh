#!/bin/bash
# Run the packaged helper and a harmless two-reader fixture in a disposable kernel.
set -euo pipefail
repo=$(cd "$(dirname "$0")/../.." && pwd)
package=$(realpath "${1:?supply the Defender deb}")
kernel=$(realpath "${2:?supply the Linux image deb}")
mkdir -p "$repo/target"
work=$(mktemp -d "$repo/target/credential-test.XXXXXX")
trap 'rm -rf -- "$work"' EXIT
cd "$work"
mkdir -p root/{bin,proc,sys,dev,etc,tmp,opt/ram-monitor}
dpkg-deb --fsys-tarfile "$kernel" | tar -xO --wildcards './boot/vmlinuz-*' > vmlinuz
dpkg-deb --fsys-tarfile "$package" | tar -xO './opt/ram-monitor/atom-credential-guard' >root/opt/ram-monitor/atom-credential-guard
chmod 755 root/opt/ram-monitor/atom-credential-guard
cp /bin/busybox root/bin/
cp /usr/bin/setpriv root/bin/test-run-as
while read -r lib; do cp -L --parents "$lib" root; done < <(
    ldd root/opt/ram-monitor/atom-credential-guard /usr/bin/setpriv |
        awk '/=> \//{print $3} /^[[:space:]]*\/lib/{print $1}' | sort -u
)
printf '%s\n' 'root:x:0:0:root:/:/bin/sh' 'tester:x:1000:1000:test:/tmp:/bin/sh' >root/etc/passwd
install -m755 "$repo/experiments/credential-guard/guest-init.sh" root/init
(cd root && find . | cpio -o --quiet -H newc --owner=0:0 | gzip -1) > initrd.gz
timeout 40 qemu-system-x86_64 -enable-kvm -cpu host -m 512 -nodefaults -display none -serial stdio -no-reboot -kernel vmlinuz -initrd initrd.gz -append 'console=ttyS0 rdinit=/init quiet panic=-1' | tee serial.log
grep -q CREDENTIAL_TEST_RESULT=PASS serial.log
