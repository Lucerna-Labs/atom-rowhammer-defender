#!/bin/busybox sh
/bin/busybox --install -s /bin
export PATH=/bin
mount -t proc proc /proc
mount -t devtmpfs dev /dev
mount -t sysfs sys /sys
exec </dev/console >/dev/console 2>&1
finish() { echo "CREDENTIAL_TEST_RESULT=$1"; poweroff -f; }
set -e
trap 'finish FAIL' EXIT
mkdir /tmp/test
cd /tmp/test
chmod 777 .
echo CANARY > secret
chown 1000:1000 secret
ln secret alias
mkfifo control trigger
chmod 666 trigger
exec 3<>control
test-run-as --reuid=1000 --regid=1000 --clear-groups /bin/sh -c 'echo ready >ready; read signal <trigger; read value <secret && echo "$value" >approved; read signal <trigger' &
reader=$!
for i in $(seq 1 100); do test -f ready && break; sleep .02; done
PKEXEC_UID=1000 /opt/ram-monitor/atom-credential-guard /tmp/test/secret "$reader" <control >receipt 2>error &
guard=$!
waitfor() { for i in $(seq 1 100); do if "$@"; then return 0; fi; sleep .02; done; cat receipt error; return 1; }
waitfor grep -q '^ARMED' receipt
for path in secret alias; do
  if test-run-as --reuid=1000 --regid=1000 --clear-groups /bin/sh -c 'read v <"$1" && echo "$v"' sh "$path" >denied 2>denial; then exit 1; fi
  grep -qi 'permission denied\|operation not permitted' denial
  test ! -s denied
done
echo go >trigger
waitfor test -s approved
test "$(cat approved)" = CANARY
for i in $(seq 1 16); do cat secret >/dev/null 2>&1 && exit 1; done
waitfor grep -q 'allowed=1 denied=18 .*tripwire=true' receipt
cat receipt
echo stop >&3
wait "$guard"
test "$(cat secret)" = CANARY
echo 'PASS approved reader, first-open denial and zero bytes, hard link, Atom trip, disarm'
trap - EXIT
finish PASS
