//! Linux libpcap capture; this executable has no packet injection or rule APIs.
use crate::engine::{ETHERNET, SLL, SLL2};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs::{File, Metadata, OpenOptions};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;
use std::ptr;
type Handle = *mut c_void;
#[repr(C)]
struct Header {
    _ts: libc::timeval,
    captured: u32,
    wire: u32,
}
#[repr(C)]
#[derive(Default)]
struct Stats {
    received: u32,
    dropped: u32,
    interface_dropped: u32,
}
#[link(name = "libpcap.so.0.8", modifiers = "+verbatim")]
unsafe extern "C" {
    fn pcap_open_live(
        name: *const c_char,
        snap: c_int,
        promisc: c_int,
        timeout: c_int,
        err: *mut c_char,
    ) -> Handle;
    fn pcap_open_offline(path: *const c_char, err: *mut c_char) -> Handle;
    fn pcap_set_datalink(p: Handle, dlt: c_int) -> c_int;
    fn pcap_setnonblock(p: Handle, on: c_int, err: *mut c_char) -> c_int;
    fn pcap_get_selectable_fd(p: Handle) -> c_int;
    fn pcap_datalink(p: Handle) -> c_int;
    fn pcap_stats(p: Handle, s: *mut Stats) -> c_int;
    fn pcap_next_ex(p: Handle, h: *mut *const Header, data: *mut *const u8) -> c_int;
    fn pcap_geterr(p: Handle) -> *const c_char;
    fn pcap_close(p: Handle);
}
pub enum Packet<'a> {
    Idle,
    End,
    Truncated,
    Data(&'a [u8]),
}
pub struct Capture {
    handle: Handle,
    pub link: i32,
    poll_fd: Option<c_int>,
    source: Option<(File, Metadata)>,
}
impl Capture {
    fn wrap(handle: Handle, err: &[c_char]) -> Result<Self, String> {
        if handle.is_null() {
            return Err(unsafe { CStr::from_ptr(err.as_ptr()) }
                .to_string_lossy()
                .into_owned());
        }
        Ok(Self {
            handle,
            link: unsafe { pcap_datalink(handle) },
            poll_fd: None,
            source: None,
        })
    }
    pub fn live() -> Result<Self, String> {
        let mut err = [0; 256];
        let mut c = Self::wrap(
            unsafe { pcap_open_live(c"any".as_ptr(), 262144, 0, 50, err.as_mut_ptr()) },
            &err,
        )?;
        unsafe {
            let _ = pcap_set_datalink(c.handle, SLL2);
            c.link = pcap_datalink(c.handle);
            if !matches!(c.link, SLL | SLL2) {
                return Err("live capture has no cooked direction metadata".into());
            }
            if pcap_setnonblock(c.handle, 1, err.as_mut_ptr()) < 0 {
                return Err(c.error());
            }
            let fd = pcap_get_selectable_fd(c.handle);
            if fd < 0 {
                return Err("capture descriptor unavailable".into());
            }
            c.poll_fd = Some(fd);
        }
        Ok(c)
    }
    pub fn offline(path: &Path) -> Result<Self, String> {
        let fd = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
            .map_err(|e| e.to_string())?;
        let meta = fd.metadata().map_err(|e| e.to_string())?;
        if !meta.is_file() {
            return Err("select a regular capture file".into());
        }
        let name =
            CString::new(format!("/proc/self/fd/{}", fd.as_raw_fd())).map_err(|e| e.to_string())?;
        let mut err = [0; 256];
        let mut c = Self::wrap(
            unsafe { pcap_open_offline(name.as_ptr(), err.as_mut_ptr()) },
            &err,
        )?;
        if !matches!(c.link, SLL | SLL2 | ETHERNET) {
            return Err(format!("unsupported capture link type {}", c.link));
        }
        c.source = Some((fd, meta));
        Ok(c)
    }
    pub fn verify_source(&self) -> Result<(), String> {
        if let Some((fd, old)) = &self.source {
            let new = fd.metadata().map_err(|e| e.to_string())?;
            if (new.len(), new.ctime(), new.ctime_nsec())
                != (old.len(), old.ctime(), old.ctime_nsec())
            {
                return Err("capture changed during replay".into());
            }
        }
        Ok(())
    }
    fn error(&self) -> String {
        unsafe { CStr::from_ptr(pcap_geterr(self.handle)) }
            .to_string_lossy()
            .into_owned()
    }
    pub fn drops(&self) -> Result<(u32, u32), String> {
        if self.poll_fd.is_none() {
            return Ok((0, 0));
        }
        let mut s = Stats::default();
        if unsafe { pcap_stats(self.handle, &mut s) } < 0 {
            return Err(self.error());
        }
        Ok((s.dropped, s.interface_dropped))
    }
    pub fn next(&mut self) -> Result<Packet<'_>, String> {
        let mut h = ptr::null();
        let mut data = ptr::null();
        match unsafe { pcap_next_ex(self.handle, &mut h, &mut data) } {
            1 => {
                if h.is_null() || data.is_null() {
                    return Err("invalid capture record".into());
                }
                let h = unsafe { &*h };
                if h.captured > h.wire || h.captured > 262144 {
                    return Err("invalid packet length".into());
                }
                if h.captured < h.wire {
                    return Ok(Packet::Truncated);
                }
                Ok(Packet::Data(unsafe {
                    std::slice::from_raw_parts(data, h.captured as usize)
                }))
            }
            0 => {
                if let Some(fd) = self.poll_fd {
                    let mut p = libc::pollfd {
                        fd,
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    let result = unsafe { libc::poll(&mut p, 1, 50) };
                    if result < 0
                        && std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted
                    {
                        return Err("capture polling failed".into());
                    }
                }
                Ok(Packet::Idle)
            }
            -2 => Ok(Packet::End),
            _ => Err(self.error()),
        }
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        unsafe { pcap_close(self.handle) }
    }
}
