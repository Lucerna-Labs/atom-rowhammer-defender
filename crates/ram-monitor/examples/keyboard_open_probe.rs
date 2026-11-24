//! Installation acceptance probe for the normal helper, parser and Spiderweb transport.
//! Waits for an OPEN from a separately run open/close test process.
use ram_monitor::keyboard::{Event, Observer};
use ram_monitor_bus::{BusEvent, GuardBus};
use std::io::{self, Write};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut observer = Observer::start()?;
    let mut bus = GuardBus::start_named(
        "defender.keyboard",
        "defender.keyboard.open",
        move |delay| observer.next(delay),
        Duration::from_millis(150),
        None,
        || {},
    )?;
    loop {
        match bus.events().recv_timeout(Duration::from_secs(8))? {
            BusEvent::Delivery(d) => match d.outcome? {
                Event::Ready => println!("READY"),
                Event::Open(o) => {
                    println!(
                        "OBSERVED {} {} {} {} {}",
                        o.pid, o.uid, o.device, o.executable, o.at
                    );
                    break;
                }
                Event::Lost(n) => return Err(format!("lost {n} events").into()),
                Event::Idle => {}
            },
            BusEvent::Crashed(s) => return Err(s.into()),
        }
        io::stdout().flush()?;
    }
    bus.stop();
    bus.join()?;
    Ok(())
}
