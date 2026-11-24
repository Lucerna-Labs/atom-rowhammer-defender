//! Controlled event-source exercise over generated files only.
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

use ram_monitor::file_events::{EventSignal, FileEventSource};
use ram_monitor::integrity_watch::{IntegrityWatch, WatchCheck};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let reference = args
        .next()
        .map(PathBuf::from)
        .ok_or("missing reference path")?;
    let observed = args
        .next()
        .map(PathBuf::from)
        .ok_or("missing observed path")?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let mut events = FileEventSource::new(&observed)?;
    let mut watch = IntegrityWatch::start_with_object(&reference, &observed)?;
    println!("ready path={}", events.selected_path().display());
    io::stdout().flush()?;
    for command in io::stdin().lock().lines() {
        if command? != "wait" {
            return Err("expected wait command".into());
        }
        let signal = events.wait(Duration::from_millis(250))?;
        match signal {
            EventSignal::Timeout => println!("signal=timeout"),
            EventSignal::WritePending { mask } => println!("signal=pending_write mask={mask:#x}"),
            EventSignal::Changed { mask } | EventSignal::Uncertain { mask } => {
                let kind = if matches!(signal, EventSignal::Changed { .. }) {
                    "changed"
                } else {
                    "uncertain"
                };
                let checked = watch.poll();
                match checked {
                    WatchCheck::ComparedWithObject(receipt, changed) => println!(
                        "signal={kind} mask={mask:#x} alert={} different={} object_changed={changed}",
                        receipt.different_bytes > 0 || *changed,
                        receipt.different_bytes
                    ),
                    WatchCheck::ReadError(error) => {
                        println!("signal={kind} mask={mask:#x} alert=true read_error={error:?}")
                    }
                    WatchCheck::EngineError(error) => {
                        println!("signal={kind} mask={mask:#x} alert=true engine_error={error:?}")
                    }
                    WatchCheck::Compared(_) => return Err("object mode unexpectedly absent".into()),
                }
            }
        }
        io::stdout().flush()?;
    }
    Ok(())
}
