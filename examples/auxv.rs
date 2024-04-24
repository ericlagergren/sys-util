#![no_std]
#![cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "solaris",
))]

use core::{
    ffi::{c_int, c_void},
    fmt::{self, Write},
};

use sys_auxv::AuxVec;

fn main() {
    let auxv = AuxVec::from_static();
    let _ = writeln!(Stdout, "{auxv:#}");
}

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        extern "C" {
            fn write(filedes: c_int, buf: *const c_void, nbyte: usize) -> c_int;
        }
        let mut buf = s.as_bytes();
        let mut nw = 0;
        while !buf.is_empty() {
            // SAFETY: FFI call, no invariants.
            let ret = unsafe { write(1, buf.as_ptr().cast(), buf.len()) };
            if ret < 0 {
                return Err(fmt::Error);
            }
            buf = &mut buf[ret as usize..]
        }
        Ok(())
    }
}
