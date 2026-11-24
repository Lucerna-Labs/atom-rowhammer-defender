//! Controlled automatic-restoration exercise over generated files only.
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

use ram_monitor::auto_restore::{AutomaticFileGuard, GuardOutcome};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let reference = args.next().map(PathBuf::from).ok_or("missing reference")?;
    let observed = args.next().map(PathBuf::from).ok_or("missing observed")?;
    let evidence = args.next().map(PathBuf::from).ok_or("missing evidence")?;
    if args.next().is_some() {
        return Err("extra argument".into());
    }
    let mut guard = AutomaticFileGuard::arm(&reference, &observed, &evidence)?;
    println!("ready");
    io::stdout().flush()?;
    for command in io::stdin().lock().lines() {
        if command? != "wait" {
            return Err("expected wait command".into());
        }
        match guard.wait_and_act(Duration::from_millis(250)) {
            Ok(GuardOutcome::Clean { signal }) => println!("clean signal={signal:?}"),
            Ok(GuardOutcome::PendingWrite) => println!("pending_write awaiting_close"),
            Ok(GuardOutcome::Restored(receipt)) => println!(
                "restored selected={:?} source={:?} executed={:?} first_failed={} evidence={} changed={} object_changed={} learned={} experiences={} elapsed_us={}",
                receipt.selected.method,
                receipt.selected.source,
                receipt.executed,
                receipt.first_attempt_failed,
                receipt.evidence.display(),
                receipt.changed_bytes,
                receipt.object_changed,
                receipt.cognitive_recorded,
                guard.successful_experiences(),
                receipt.elapsed.as_micros()
            ),
            Err(error) => println!("error={error}"),
        }
        io::stdout().flush()?;
    }
    Ok(())
}
