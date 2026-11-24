//! A Spiderweb-owned worker for one armed file guard. The caller's tick
//! closure performs the complete safety transaction before bus publication.
use std::fmt;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use spiderweb::{
    register_clone_type, Bus, BusHandle, LifecycleEvent, Socket, Strand, StrandError, SYS_LIFECYCLE,
};

pub const GUARD_SOCKET: &str = "defender.guard.outcome";
const GUARD_STRAND: &str = "defender.guard";

#[derive(Clone, Debug)]
pub struct GuardDelivery<T> {
    pub check: u64,
    pub outcome: Result<T, String>,
}

#[derive(Clone, Debug)]
pub enum BusEvent<T> {
    Delivery(GuardDelivery<T>),
    Crashed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BusReport {
    pub published: u64,
    pub received: u64,
    pub crashed: u64,
    pub detached: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BusRuntimeError {
    Spawn(String),
    WorkerPanicked,
    AlreadyJoined,
    MissingDelivery { published: u64, received: u64 },
    StrandCrashed(u64),
    TaskFailed(String),
    Output(String),
}
impl fmt::Display for BusRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(value) => write!(f, "Spiderweb worker spawn: {value}"),
            Self::WorkerPanicked => f.write_str("Spiderweb runtime panicked"),
            Self::AlreadyJoined => f.write_str("Spiderweb runtime already joined"),
            Self::MissingDelivery { published, received } => write!(
                f,
                "Spiderweb guard outcome delivery mismatch: published {published}, received {received}"
            ),
            Self::StrandCrashed(count) => write!(f, "Spiderweb guard strand crashed {count} time(s)"),
            Self::TaskFailed(value) => f.write_str(value),
            Self::Output(value) => write!(f, "guard output: {value}"),
        }
    }
}
impl std::error::Error for BusRuntimeError {}

type Tick<T> = Box<dyn FnMut(Duration) -> Result<T, String> + Send>;

struct GuardStrand<T> {
    name: &'static str,
    socket: &'static str,
    tick: Tick<T>,
    interval: Duration,
    limit: Option<u64>,
    checks: u64,
    stop: Arc<AtomicBool>,
    published: Arc<AtomicU64>,
    outputs: [Socket; 1],
}

impl<T: Clone + Send + Sync + 'static> Strand for GuardStrand<T> {
    fn name(&self) -> &str {
        self.name
    }
    fn inputs(&self) -> &[Socket] {
        &[]
    }
    fn outputs(&self) -> &[Socket] {
        &self.outputs
    }
    fn run(&mut self, bus: &mut BusHandle) -> Result<(), StrandError> {
        if self.stop.load(Ordering::Acquire) || bus.is_shutting_down() {
            return Err(StrandError::Detach);
        }
        self.checks = self.checks.saturating_add(1);
        let outcome = (self.tick)(self.interval);
        let terminal = outcome.is_err() || self.limit.is_some_and(|n| self.checks >= n);
        bus.publish_static(
            self.socket,
            GuardDelivery {
                check: self.checks,
                outcome,
            },
        )
        .map_err(|error| StrandError::Crashed(error.to_string()))?;
        self.published.fetch_add(1, Ordering::Release);
        if terminal {
            Err(StrandError::Detach)
        } else {
            Ok(())
        }
    }
}

pub struct GuardBus<T> {
    stop: Arc<AtomicBool>,
    events: Receiver<BusEvent<T>>,
    worker: Option<JoinHandle<spiderweb::BusStats>>,
    published: Arc<AtomicU64>,
    received: Arc<AtomicU64>,
}

impl<T: Clone + Send + Sync + 'static> GuardBus<T> {
    /// Subscribe before registering the live strand, which starts immediately.
    pub fn start(
        tick: impl FnMut(Duration) -> Result<T, String> + Send + 'static,
        interval: Duration,
        limit: Option<u64>,
    ) -> Result<Self, BusRuntimeError> {
        Self::start_with_notify(tick, interval, limit, || {})
    }

    pub fn start_with_notify(
        tick: impl FnMut(Duration) -> Result<T, String> + Send + 'static,
        interval: Duration,
        limit: Option<u64>,
        notify: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, BusRuntimeError> {
        Self::start_named(GUARD_STRAND, GUARD_SOCKET, tick, interval, limit, notify)
    }

    pub fn start_named(
        name: &'static str, socket: &'static str,
        tick: impl FnMut(Duration) -> Result<T, String> + Send + 'static,
        interval: Duration, limit: Option<u64>,
        notify: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, BusRuntimeError> {
        if interval.is_zero() || interval > Duration::from_secs(60) || limit == Some(0) {
            return Err(BusRuntimeError::Spawn(
                "invalid interval or check limit".into(),
            ));
        }
        register_clone_type::<GuardDelivery<T>>();
        let mut bus = Bus::open();
        let publisher = bus.publisher();
        let (sender, events) = channel();
        let received = Arc::new(AtomicU64::new(0));
        let delivery_sender = sender.clone();
        let delivery_count = Arc::clone(&received);
        publisher.subscribe_exact(
            socket,
            Box::new(move |_, payload| {
                if let Ok(delivery) = payload.downcast::<GuardDelivery<T>>() {
                    if delivery_sender.send(BusEvent::Delivery(*delivery)).is_ok() {
                        delivery_count.fetch_add(1, Ordering::Release);
                        notify();
                    }
                }
            }),
        );
        publisher.subscribe_exact(
            SYS_LIFECYCLE,
            Box::new(move |_, payload| {
                if let Ok(LifecycleEvent::Crashed { name: crashed_name, reason }) =
                    payload.downcast::<LifecycleEvent>().map(|event| *event)
                {
                    if crashed_name == name {
                        let _ = sender.send(BusEvent::Crashed(reason));
                    }
                }
            }),
        );
        let stop = Arc::new(AtomicBool::new(false));
        let published = Arc::new(AtomicU64::new(0));
        bus.register(GuardStrand {
            name, socket,
            tick: Box::new(tick),
            interval,
            limit,
            checks: 0,
            stop: Arc::clone(&stop),
            published: Arc::clone(&published),
            outputs: [Socket::new::<GuardDelivery<T>>(socket)],
        });
        let worker = thread::Builder::new()
            .name("ram-monitor-spiderweb".into())
            .spawn(move || {
                bus.run_until(None);
                bus.stats()
            })
            .map_err(|error| {
                stop.store(true, Ordering::Release);
                BusRuntimeError::Spawn(error.to_string())
            })?;
        Ok(Self {
            stop,
            events,
            worker: Some(worker),
            published,
            received,
        })
    }

    pub fn events(&self) -> &Receiver<BusEvent<T>> {
        &self.events
    }
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
    }
    pub fn is_finished(&self) -> bool {
        self.worker.as_ref().is_some_and(JoinHandle::is_finished)
    }
    pub fn join(&mut self) -> Result<BusReport, BusRuntimeError> {
        let worker = self.worker.take().ok_or(BusRuntimeError::AlreadyJoined)?;
        let stats = worker.join().map_err(|_| BusRuntimeError::WorkerPanicked)?;
        let report = BusReport {
            published: self.published.load(Ordering::Acquire),
            received: self.received.load(Ordering::Acquire),
            crashed: stats.crashed,
            detached: stats.detached,
        };
        verify(report)?;
        Ok(report)
    }
}

fn verify(report: BusReport) -> Result<(), BusRuntimeError> {
    if report.published != report.received {
        return Err(BusRuntimeError::MissingDelivery {
            published: report.published,
            received: report.received,
        });
    }
    if report.crashed > 0 {
        return Err(BusRuntimeError::StrandCrashed(report.crashed));
    }
    Ok(())
}

/// Consume the typed bus subscription as the CLI's authoritative output path.
pub fn run_to_stdout<T: Clone + Send + Sync + 'static>(
    tick: impl FnMut(Duration) -> Result<T, String> + Send + 'static,
    interval: Duration,
    limit: Option<u64>,
    is_pending: impl Fn(&T) -> bool,
    format_line: impl Fn(u64, &T) -> String,
) -> Result<BusReport, BusRuntimeError> {
    let mut runtime = GuardBus::start(tick, interval, limit)?;
    let mut terminal_error = None;
    let mut pending = false;
    for event in runtime.events().iter() {
        match event {
            BusEvent::Delivery(GuardDelivery {
                check,
                outcome: Ok(value),
            }) => {
                pending = is_pending(&value);
                println!("{}", format_line(check, &value));
                io::stdout()
                    .flush()
                    .map_err(|error| BusRuntimeError::Output(error.to_string()))?;
            }
            BusEvent::Delivery(GuardDelivery {
                check,
                outcome: Err(error),
            }) => {
                terminal_error = Some(format!("guard_check={check} {error}"));
            }
            BusEvent::Crashed(error) => {
                terminal_error = Some(format!("guard strand crashed: {error}"))
            }
        }
    }
    let report = runtime.join()?;
    if let Some(error) = terminal_error {
        return Err(BusRuntimeError::TaskFailed(error));
    }
    if pending {
        return Err(BusRuntimeError::TaskFailed(
            "selected file write remained open at the final guard check".into(),
        ));
    }
    Ok(report)
}

impl<T> Drop for GuardBus<T> {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_strand_delivers_each_typed_outcome() {
        let mut bus =
            GuardBus::start(|_| Ok(7usize), Duration::from_millis(1), Some(3)).expect("start");
        let mut checks = Vec::new();
        while let Ok(event) = bus.events().recv() {
            if let BusEvent::Delivery(delivery) = event {
                checks.push((delivery.check, delivery.outcome.expect("tick")));
            }
        }
        let report = bus.join().expect("join");
        assert_eq!(checks, [(1, 7), (2, 7), (3, 7)]);
        assert_eq!(
            (report.published, report.received, report.crashed),
            (3, 3, 0)
        );
    }

    #[test]
    fn missing_subscriber_is_an_error_even_when_publish_succeeds() {
        let bus = Bus::open();
        let publisher = bus.publisher();
        register_clone_type::<GuardDelivery<usize>>();
        assert!(publisher
            .publish(
                GUARD_SOCKET,
                GuardDelivery {
                    check: 1,
                    outcome: Ok(7usize)
                }
            )
            .is_ok());
        assert_eq!(
            verify(BusReport {
                published: 1,
                received: 0,
                crashed: 0,
                detached: 0
            }),
            Err(BusRuntimeError::MissingDelivery {
                published: 1,
                received: 0
            })
        );
    }

    #[test]
    fn strand_panic_reports_crash_and_stops() {
        let mut bus = GuardBus::<usize>::start(
            |_| panic!("forced guard crash"),
            Duration::from_millis(1),
            None,
        )
        .expect("start");
        let mut saw_crash = false;
        while let Ok(event) = bus.events().recv() {
            saw_crash |= matches!(event, BusEvent::Crashed(_));
        }
        assert!(saw_crash);
        assert_eq!(bus.join(), Err(BusRuntimeError::StrandCrashed(1)));
    }
}
