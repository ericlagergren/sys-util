#![no_std]
#![no_main]
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

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let auxv = AuxVec::from_static();
    for _ in 0..100 {
        let _ = writeln!(Stdout, "{auxv:#}");
    }
    0
}

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        extern "C" {
            fn write(filedes: c_int, buf: *const c_void, nbyte: usize) -> c_int;
        }
        let mut buf = s.as_bytes();
        while !buf.is_empty() {
            // SAFETY: FFI call, no invariants.
            let ret = unsafe { write(1, buf.as_ptr().cast(), buf.len()) };
            if ret < 0 {
                return Err(fmt::Error);
            }
            buf = &buf[ret as usize..]
        }
        Ok(())
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
