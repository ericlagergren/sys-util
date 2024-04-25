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
    ffi::{c_char, c_int, c_void},
    fmt::{self, Write},
};

use sys_auxv::AuxVec;

unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    for i in 0..n {
        dst.add(i).write_volatile(src.read_volatile())
    }
    dst
}

#[no_mangle]
pub extern "C" fn main(_argc: c_int, _argv: *const *const c_char) -> c_int {
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

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
