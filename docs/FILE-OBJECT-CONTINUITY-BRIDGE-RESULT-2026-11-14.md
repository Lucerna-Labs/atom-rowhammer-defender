# File-object continuity — read-only source-built observation

Frozen bridge protocol:
`FILE-OBJECT-CONTINUITY-BRIDGE-PRECOMMIT-2026-11-14.md` at `d6693d2`.
The app now exposes a conventional, separate file-object sensor through
`--source object-capture` and `--source object-check`. Capture returns
a strict `device:inode:birth_ns` token plus link count and size. Check
accepts that user-held token and reports whether the regular file now
reached by a pathname is the same object. The GUI has explicit
**Capture identity** and **Check object** buttons with an in-memory
baseline. Neither action reads content, watches a path, modifies a file,
or changes Rowhammer ON/OFF.

The new real ext4 CLI integration test created its own 160-byte sandbox
file. Same-path repeat, rename, hardlink and the old hardlink after
replacement retained the captured object identity. Identical-byte
copy and atomic rename-over replacement produced a different identity.
A symlink followed the currently reached replacement target. The
unchanged E012/E013 Atom content comparison on the trusted reference
and observed file had zero conventional byte differences and an
**identical complete material receipt** before and after the same-content
replacement. This directly demonstrates the information missing from
a content-only input. Invalid token shapes/characters/overflow,
missing files, directory paths and missing or incompatible CLI options
were rejected without a success observation. The test removed only its
generated sandbox.

The source-built GUI was launched on isolated Xvfb with Rowhammer OFF.
It visibly reported **Same file object** after capture and check.
An `os.replace` of a staged, identically sized 160-byte file over the
same path then made it visibly report **Different file object** while
Rowhammer remained OFF. The bytes had identical SHA-256
`d5d93b11c70a0c80c427e20a163097f227f8704e3793b30c6086a75b41ac789e`.
The old and new object tokens are in
`experiments/file-object-app/GUI-REPLACEMENT.json`, SHA-256
`0c7ce1fc89b53235aabeed4655b91cc0c819ce96a5cbc7b2847045960f74f1b6`.
Screenshots `gui/same.png` and `gui/different.png` have SHA-256
`1ab4edec9f2115df138c9c37a299c2a08e9c3984deed55bbd5167daac4326b81`
and `16c506dc7d86f6ec845018310aa7a0b031a1b58c8b4287f12d39a33abb933ec0`.
The generated GUI sandbox was verified and removed. The full locked
offline Rust workspace suite and portable-only guard passed.

**Boundary:** this is a point-in-time conventional supporting sensor,
not another Atom stack or proof of unauthorized activity. An in-place
edit keeps object identity; a copy or legitimate atomic application
update changes it. The user-held token is not authenticated or fresh,
and device/inode/birth time cannot prove who made a change. Birth time
availability varies by filesystem; the app fails closed when absent.
Source-built Xvfb evidence is not an installed `/opt/ram-monitor` or
live COSMIC-window test. No automatic watcher or remediation was added.

Filesystem basis: [inode(7)](https://man7.org/linux/man-pages/man7/inode.7.html),
[statx(2)](https://man7.org/linux/man-pages/man2/statx.2.html), and
[rename(2)](https://man7.org/linux/man-pages/man2/rename.2.html).
