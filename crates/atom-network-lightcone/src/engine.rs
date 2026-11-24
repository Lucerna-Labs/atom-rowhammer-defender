//! The native Door One strand owns every decision and sequenced receipt.
use crate::{Receipt, Report};
use imperium_net_lightcone::{
    hash, hex, topics, FaultPolicy, LightconeService, LightconeStrand, WireDirection,
};
use imperium_net_pathworld::PathWorld;
use imperium_protocol::{
    BusEvent, LightconeFrameFormat, LightconeReceipt, LightconeStructure, Message,
};
use spiderweb::{Bus, BusPublisher};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::{mpsc, Arc};
use std::time::Duration;

pub const ETHERNET: i32 = 1;
pub const SLL: i32 = 113;
pub const SLL2: i32 = 276;
const MAX_PACK: u64 = 48 * 1024 * 1024;
const WINDOW: usize = 256;

pub struct Engine {
    bus: Bus,
    publisher: BusPublisher,
    receipts: mpsc::Receiver<LightconeReceipt>,
    pending: VecDeque<String>,
    pub report: Report,
}

impl Engine {
    pub fn load(file: &Path) -> Result<Self, String> {
        let fd = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(file)
            .map_err(|e| e.to_string())?;
        let meta = fd.metadata().map_err(|e| e.to_string())?;
        if !meta.is_file() || meta.len() > MAX_PACK {
            return Err("select a regular path pack up to 48 MiB".into());
        }
        let mut src = String::new();
        fd.take(MAX_PACK + 1)
            .read_to_string(&mut src)
            .map_err(|e| e.to_string())?;
        if src.len() as u64 > MAX_PACK {
            return Err("path pack exceeded 48 MiB".into());
        }
        let mut header = src.lines();
        header.next().ok_or("incomplete pack header")?;
        let role = header
            .next()
            .and_then(|s| s.strip_prefix("role "))
            .ok_or("missing pack role")?;
        if role.len() > 128 || role.chars().any(char::is_control) {
            return Err("invalid pack role".into());
        }
        let count = header
            .next()
            .and_then(|s| s.strip_prefix("paths "))
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or("invalid path count")?;
        if !(1..=500_000).contains(&count) {
            return Err("select a pack with 1 to 500,000 reference paths".into());
        }
        let mut symbols = 0usize;
        for (i, line) in header.enumerate() {
            // Bound allocations and native trie recursion before handing untrusted input to the loader.
            let n = line.split(' ').count();
            symbols += n;
            if i >= count || n > 64 || symbols > 4_000_000 {
                return Err("path pack exceeds structural limits".into());
            }
        }
        let digest = hex(&hash(src.as_bytes()));
        let world =
            PathWorld::from_pack(&src).map_err(|e| format!("invalid reference pack: {e:?}"))?;
        if world.folds() != 0 {
            return Err("expected an exact-path world".into());
        }
        let service = Arc::new(LightconeService::with_world(world));
        let info = service.info();
        let report = Report {
            role: info.world,
            pack_sha256: digest,
            trained_paths: info.trained_paths,
            deny_paths: info.deny_paths,
            ..Default::default()
        };
        let mut bus = Bus::open();
        let publisher = bus.publisher();
        let (tx, receipts) = mpsc::channel();
        publisher.subscribe_exact(
            topics::RECEIPT,
            Box::new(move |_, item| {
                if let Ok(event) = item.downcast::<BusEvent>() {
                    if let Message::LightconeReceipt(r) = event.message {
                        let _ = tx.send(r);
                    }
                }
            }),
        );
        bus.register(LightconeStrand::new(service, FaultPolicy::None));
        Ok(Self {
            bus,
            publisher,
            receipts,
            pending: VecDeque::new(),
            report,
        })
    }
    fn accept(&mut self, r: LightconeReceipt) -> Result<(), String> {
        if self.pending.pop_front().as_deref() != Some(&r.query_sha256)
            || r.sequence != self.report.samples + 1
            || !r.passive
            || r.holds_verdict
            || !r.learned_positions.is_empty()
        {
            return Err("native receipt sequence, hash or exact-path invariant failed".into());
        }
        let direction = match r.direction {
            0 => {
                self.report.inbound += 1;
                "inbound"
            }
            1 => {
                self.report.outbound += 1;
                "outbound"
            }
            _ => return Err("invalid receipt direction".into()),
        };
        let kind = match r.observation_kind {
            0 => "decoded",
            1 => "unknown",
            2 => "malformed",
            3 => "fragment",
            _ => return Err("invalid receipt kind".into()),
        };
        self.report.samples = r.sequence;
        self.report.matched += u64::from(r.admitted);
        self.report.denied += u64::from(r.denied);
        self.report.unrepresented += u64::from(r.path.is_empty());
        self.report.last = Some(Receipt {
            sequence: r.sequence,
            direction: direction.into(),
            kind: kind.into(),
            path: r
                .path
                .iter()
                .map(|s| format!("{}:{:x}", s.field, s.value))
                .collect(),
            admitted: r.admitted,
            denied: r.denied,
            query_sha256: r.query_sha256,
        });
        Ok(())
    }
    fn wait_one(&mut self) -> Result<(), String> {
        let r = self
            .receipts
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| "native Lightcone receipt unavailable")?;
        self.accept(r)
    }
    pub fn flush(&mut self) -> Result<(), String> {
        while !self.pending.is_empty() {
            self.wait_one()?;
        }
        Ok(())
    }
    pub fn observe(
        &mut self,
        link: i32,
        data: &[u8],
        ether_direction: Option<WireDirection>,
    ) -> Result<(), String> {
        let Some(structure) = structure(link, data, ether_direction)? else {
            self.report.missing_link_metadata += 1;
            return Ok(());
        };
        if self.pending.len() >= WINDOW {
            self.wait_one()?;
        }
        self.pending.push_back(hex(&structure.query_hash));
        BusEvent::publish(
            &self.publisher,
            topics::FRAME,
            Message::LightconeStructure(structure),
        )
        .map_err(|e| e.to_string())?;
        while let Ok(r) = self.receipts.try_recv() {
            self.accept(r)?;
        }
        Ok(())
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        self.bus.run_until(Some(Duration::ZERO));
    }
}

fn structure(
    link: i32,
    frame: &[u8],
    ether_direction: Option<WireDirection>,
) -> Result<Option<LightconeStructure>, String> {
    use LightconeFrameFormat::Ethernet;
    if link == ETHERNET {
        let direction = ether_direction.ok_or("select the direction of this Ethernet capture")?;
        return Ok(Some(LightconeService::structure_from_frame(
            direction as u8,
            Ethernet,
            frame,
            None,
        )));
    }
    let (head, protocol, packet_type) = match link {
        SLL2 if frame.len() >= 20 => (20, &frame[..2], u16::from(frame[10])),
        SLL if frame.len() >= 16 => (16, &frame[14..16], u16::from_be_bytes([frame[0], frame[1]])),
        SLL | SLL2 => return Ok(None),
        _ => return Err(format!("unsupported capture link type {link}")),
    };
    if packet_type > 4 {
        return Ok(None);
    }
    // Cooked captures omit Ethernet MACs. Supply only their observed protocol and
    // body to the Ethernet decoder; bind the receipt to the ORIGINAL capture bytes.
    let mut view = Vec::with_capacity(14 + frame.len() - head);
    view.extend_from_slice(&[0; 12]);
    view.extend_from_slice(protocol);
    view.extend_from_slice(&frame[head..]);
    let mut s =
        LightconeService::structure_from_frame(u8::from(packet_type == 4), Ethernet, &view, None);
    s.query_hash = hash(frame);
    Ok(Some(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cooked_header_keeps_direction_and_hash_of_original() {
        for (link, offset) in [(SLL, 16), (SLL2, 20)] {
            let mut b = vec![0; offset + 28];
            if link == SLL {
                b[1] = 4;
                b[14..16].copy_from_slice(&[8, 6]);
            } else {
                b[10] = 4;
                b[..2].copy_from_slice(&[8, 6]);
            }
            b[offset..offset + 8].copy_from_slice(&[0, 1, 8, 0, 6, 4, 0, 1]);
            let s = structure(link, &b, None).unwrap().unwrap();
            assert_eq!(s.direction, 1);
            assert_eq!(s.query_hash, hash(&b));
            assert_eq!(s.observation_kind, 0);
            assert_eq!(s.path[0].value, 1);
            assert!(structure(link, &b[..2], None).unwrap().is_none());
        }
        assert!(structure(ETHERNET, &[], None).is_err());
    }
}
