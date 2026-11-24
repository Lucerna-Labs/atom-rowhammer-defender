use atom_network_lightcone::{Message, Receipt, Report};
mod capture;
mod engine;
use capture::{Capture, Packet};
use engine::{Engine, ETHERNET};
use imperium_net_lightcone::WireDirection;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};

fn drop_privileges() -> Result<(), String> {
    #[repr(C)]
    struct Header {
        version: u32,
        pid: i32,
    }
    let header = Header {
        version: 0x20080522,
        pid: 0,
    };
    let empty = [0u32; 6];
    unsafe {
        if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) < 0
            || libc::syscall(libc::SYS_capset, &header, &empty) < 0
        {
            return Err(format!(
                "dropping capture capabilities: {}",
                io::Error::last_os_error()
            ));
        }
    }
    Ok(())
}
fn send(message: Message) -> Result<(), String> {
    let mut out = io::stdout().lock();
    serde_json::to_writer(&mut out, &message).map_err(|e| e.to_string())?;
    out.write_all(b"\n")
        .and_then(|_| out.flush())
        .map_err(|e| e.to_string())
}
fn stopping() -> Result<bool, String> {
    let mut p = libc::pollfd {
        fd: 0,
        events: libc::POLLIN,
        revents: 0,
    };
    let n = unsafe { libc::poll(&mut p, 1, 0) };
    if n < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
        return Err("control pipe failed".into());
    }
    Ok(p.revents != 0)
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let (live,pack,file,managed,direction)=match args.as_slice(){
        [m,p] if m=="live"=>(true,p,None,true,None),
        [m,p,c,d] if m=="replay" || m=="replay-managed" =>{
            let direction=match d.to_str() {Some("inbound")=>Some(WireDirection::Inbound),Some("outbound")=>Some(WireDirection::Outbound),
                Some("capture")=>None,_=>return Err("expected capture, inbound or outbound direction".into())};
            (false,p,Some(c.as_os_str()),m=="replay-managed",direction)
        }
        _=>return Err("usage: atom-network-lightcone live PACK | replay PACK CAPTURE capture|inbound|outbound".into())
    };
    // Only the fixed passive Linux 'any' source is opened before dropping privileges.
    let live_source = if live { Some(Capture::live()?) } else { None };
    drop_privileges()?;
    let mut source = match live_source {
        Some(c) => c,
        None => Capture::offline(Path::new(file.ok_or("missing capture")?))?,
    };
    if source.link == ETHERNET && direction.is_none() {
        return Err("Ethernet captures require an explicit inbound or outbound direction".into());
    }
    let mut engine = Engine::load(Path::new(pack))?;
    send(Message::Ready(engine.report.clone()))?;
    let link = source.link;
    let mut last_update = Instant::now();
    loop {
        if managed && stopping()? {
            break;
        }
        match source.next()? {
            Packet::Idle => {}
            Packet::End => break,
            Packet::Truncated => engine.report.truncated += 1,
            Packet::Data(data) => engine.observe(link, data, direction)?,
        }
        if last_update.elapsed() >= Duration::from_millis(350) {
            engine.flush()?;
            let (kernel, interface) = source.drops()?;
            engine.report.kernel_drops = kernel.into();
            engine.report.interface_drops = interface.into();
            send(Message::Progress(engine.report.clone()))?;
            last_update = Instant::now();
        }
    }
    engine.flush()?;
    source.verify_source()?;
    let (kernel, interface) = source.drops()?;
    engine.report.kernel_drops = kernel.into();
    engine.report.interface_drops = interface.into();
    send(Message::Complete(engine.report.clone()))
}
fn main() {
    if let Err(e) = run() {
        let _ = send(Message::Error(e));
        std::process::exit(1);
    }
}
