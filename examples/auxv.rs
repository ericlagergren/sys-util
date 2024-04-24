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
    ffi::{c_char, c_int, c_void},
    fmt,
};

use sys_auxv::AuxVec;

fn main() {
    let auxv = AuxVec::from_static();
    writeln!(Stdout, "{auxv:#}");
}

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        extern "C" {
            fn write(filedes: c_int, buf: *const c_void, nbyte: usize) -> c_int;
        }
        // SAFETY: FFI call, no invariants.
        unsafe { write(1, s.as_ptr().cast(), s.len()) }
    }
}
