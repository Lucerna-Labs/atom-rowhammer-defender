//! Fixed-purpose Linux observer. The kernel object is embedded and immutable.
//! No file path, BPF program, hook, or executable is accepted from the caller.
use std::ffi::{c_int, c_long, c_void};
use std::io::{self, Write};
use std::ptr::{null, null_mut};

type Object = *mut c_void;
#[link(name = "libbpf.so.1", modifiers = "+verbatim")]
unsafe extern "C" {
    fn bpf_object__open_mem(data: *const u8, size: usize, opts: *const c_void) -> Object;
    fn bpf_object__load(obj: Object) -> c_int;
    fn bpf_object__find_program_by_name(obj: Object, name: *const i8) -> Object;
    fn bpf_program__attach_trace(prog: Object) -> Object;
    fn bpf_object__find_map_fd_by_name(obj: Object, name: *const i8) -> c_int;
    fn bpf_map_lookup_elem(fd: c_int, key: *const u32, val: *mut u64) -> c_int;
    fn libbpf_get_error(ptr: Object) -> c_long;
    fn ring_buffer__new(
        fd: c_int,
        cb: extern "C" fn(Object, Object, usize) -> c_int,
        ctx: Object,
        opts: *const c_void,
    ) -> Object;
    fn ring_buffer__poll(ring: Object, timeout_ms: c_int) -> c_int;
    fn ring_buffer__free(ring: Object);
    fn bpf_link__destroy(link: Object) -> c_int;
    fn bpf_object__close(obj: Object);
}

struct Capture {
    obj: Object,
    ring: Object,
    link: Object,
}
impl Drop for Capture {
    fn drop(&mut self) {
        unsafe {
            if !self.link.is_null() {
                bpf_link__destroy(self.link);
            }
            if !self.ring.is_null() {
                ring_buffer__free(self.ring);
            }
            if !self.obj.is_null() {
                bpf_object__close(self.obj);
            }
        }
    }
}

fn valid(p: Object, operation: &str) -> Result<Object, String> {
    let error = unsafe { libbpf_get_error(p) };
    if error != 0 || p.is_null() {
        return Err(format!("{operation} failed ({error})"));
    }
    Ok(p)
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Event {
    time_ns: u64,
    ino: u64,
    pid: u32,
    uid: u32,
    exe_dev: u32,
    device: [u8; 24],
    executable: [u8; 64],
    reserved: u32,
}
const _: () = assert!(std::mem::size_of::<Event>() == 120);

fn safe_field(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take_while(|b| **b != 0)
        .map(|&b| {
            if b.is_ascii_alphanumeric() || b"._+-".contains(&b) {
                char::from(b)
            } else {
                '_'
            }
        })
        .collect()
}

extern "C" fn receive(_: Object, data: Object, size: usize) -> c_int {
    if data.is_null() || size != std::mem::size_of::<Event>() {
        return -libc::EPROTO;
    }
    let e = unsafe { data.cast::<Event>().read_unaligned() };
    let name = safe_field(&e.executable);
    let device = safe_field(&e.device);
    let name = if name.is_empty() { "unknown" } else { &name };
    let out = writeln!(
        io::stdout().lock(),
        "OPEN {} {} {} {} {} {} {}",
        e.time_ns,
        e.pid,
        e.uid,
        e.exe_dev,
        e.ino,
        device,
        name
    );
    if out.is_err() {
        -libc::EPIPE
    } else {
        0
    }
}

fn drop_capabilities() -> io::Result<()> {
    #[repr(C)]
    struct Header {
        version: u32,
        pid: i32,
    }
    let header = Header {
        version: 0x20080522,
        pid: 0,
    };
    let empty = [0u32; 6]; // two cap_user_data_t records
    unsafe {
        if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) < 0
            || libc::syscall(libc::SYS_capset, &header, &empty) < 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

fn run() -> Result<(), String> {
    if std::env::args_os().len() != 1 {
        return Err("this observer accepts no arguments".into());
    }
    let object = include_bytes!("../kernel/keyboard.bpf.o");
    let mut capture = Capture {
        obj: null_mut(),
        ring: null_mut(),
        link: null_mut(),
    };
    let lost_fd;
    unsafe {
        capture.obj = valid(
            bpf_object__open_mem(object.as_ptr(), object.len(), null()),
            "open embedded object",
        )?;
        let loaded = bpf_object__load(capture.obj);
        if loaded < 0 {
            return Err(format!("kernel refused keyboard observer: {}. Install the capability-limited helper and verify kernel BTF support.", io::Error::from_raw_os_error(-loaded)));
        }
        let events_fd = bpf_object__find_map_fd_by_name(capture.obj, c"events".as_ptr());
        lost_fd = bpf_object__find_map_fd_by_name(capture.obj, c"lost".as_ptr());
        if events_fd < 0 || lost_fd < 0 {
            return Err("observer maps unavailable".into());
        }
        capture.ring = valid(
            ring_buffer__new(events_fd, receive, null_mut(), null()),
            "open event channel",
        )?;
        let prog = valid(
            bpf_object__find_program_by_name(capture.obj, c"keyboard_open".as_ptr()),
            "find keyboard program",
        )?;
        capture.link = valid(bpf_program__attach_trace(prog), "attach keyboard open hook")?;
    }
    drop_capabilities().map_err(|e| format!("dropping capabilities: {e}"))?;
    let mut output = io::stdout();
    writeln!(output, "READY 1")
        .and_then(|()| output.flush())
        .map_err(|e| e.to_string())?;
    let mut previous_lost = 0;
    loop {
        // The dashboard owns the write end of stdin. Closing it detaches the hook,
        // including if the dashboard crashes while the keyboard is idle.
        let mut input = libc::pollfd {
            fd: 0,
            events: libc::POLLIN,
            revents: 0,
        };
        let poll = unsafe { libc::poll(&mut input, 1, 0) };
        if poll < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            return Err("control pipe failed".into());
        }
        if input.revents != 0 {
            return Ok(());
        }
        let n = unsafe { ring_buffer__poll(capture.ring, 200) };
        if n < 0 && n != -libc::EINTR {
            return Err(format!("keyboard event channel failed ({n})"));
        }
        let mut lost = 0;
        if unsafe { bpf_map_lookup_elem(lost_fd, &0, &mut lost) } < 0 {
            return Err("cannot verify event-loss count".into());
        }
        if lost != previous_lost {
            writeln!(output, "LOST {}", lost - previous_lost).map_err(|e| e.to_string())?;
            previous_lost = lost;
        }
        output.flush().map_err(|e| e.to_string())?;
    }
}

fn main() {
    if let Err(error) = run() {
        let _ = writeln!(io::stdout().lock(), "ERROR {error}");
        eprintln!("Keyboard Guard: {error}");
        std::process::exit(1);
    }
}
