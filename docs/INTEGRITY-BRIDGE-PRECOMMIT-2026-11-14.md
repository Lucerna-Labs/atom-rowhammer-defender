# Trusted-reference byte integrity bridge — frozen before app integration

This work imports the already measured E012/E013 Atom family as a separate
component and connects it to an exact, operator-selected file-pair boundary in
the Rowhammer Defender application. It is *not* a newly discovered Atom family
or a claim that the earlier auth/network experiments were admitted. The app
will preserve the original Rowhammer trace and GUI behavior.

Source package: `atom-stack-store/e012-statistical-integrity-memory`.
Copy only its two owning Rust crates, preserve their source bytes, Cargo names,
and provenance hashes. The source package reports 1360 fresh E013 trajectories
and 74/74 selected confirmation gates. Its admitted capability is retained
altered-byte evidence against an externally trusted 160-record reference;
it does not authenticate or protect the reference, prove freshness, infer
intent, provide a MAC, or enforce host protection.

## App boundary

CLI: explicit `--source integrity --reference-file PATH --observed-file PATH`.
GUI: a separate operator-selected pair of paths and a **Compare files** action.
Both read exactly 160 bytes from each file, reject missing/truncated/oversize
input, and run the unmodified Atom engine as `B/full/logistic/2202/off` for
320 observations. Bytes are demultiplexed low nibble then high nibble into
four paired bit coordinates, as in the source adapter. No file is modified.

The app reports four numerical material outputs per tick, their mean, the
mean over ticks 240..303 inclusive, the original-window mean over 160..223,
and the maximum mean. These are observed numbers only; the app does not turn
them into a malicious-tamper verdict or auto-revert. An exact byte-difference
count may be printed separately as a conventional comparison and must not
enter the Atom engine or be described as its emergent output.

This is a specific file-alteration observation function, not a blanket file
protector. No watcher, privileged intercept, restored copy, or live source is
part of this integration. A numerical response can support an operator review
of a trusted comparison, but cannot establish who changed a file.

## Frozen verification

1. Verify vendored Rust source SHA-256 equals the store package's provenance
   for both owning crates and their manifests. Run their native tests.
2. Run the exact stored E013 2202 reference/clean/modified byte records through
   the new Rust adapter. Clean must yield zero; modified must yield a long mean
   above `0.001`; the modified long mean must match the direct native engine
   result at fixed seed 2202 within `1e-11`.
3. Run the other three E013 altered profiles at the same fixed seed, plus
   a single-record change and identical-content replay. Report every response;
   do not adjust the frozen seed or threshold for a weak case.
4. Reject zero, 159, and 161-byte inputs and missing paths. Exercise CLI with
   real Disk2 test files, verify nonzero failure on malformed input, and run
   existing Rowhammer trace/replay tests and portable-only guard unchanged.
5. Compile GUI and exercise the pair action if a graphical desktop surface is
   available; distinguish compile evidence from visible runtime evidence.

Measured numerical differences do not make this a physical quantum or
cryptographic protection claim. The rights holder subsequently authorized MIT distribution of the bundled source.
