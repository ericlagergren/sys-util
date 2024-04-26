#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(lang_items)]
#![cfg(all(target_os = "freebsd", target_arch = "x86_64"))]

use core::{
    arch::asm,
    ffi::{c_char, c_int, c_void},
    fmt::{self, Write},
};

use sys_auxv::AuxVec;

const SYS_EXIT: i64 = 1;
const SYS_WRITE: i64 = 1;

type RawPtr = *mut c_void;

#[repr(transparent)]
struct Arg(RawPtr);

impl Arg {
    const fn none() -> Self {
        Self(0 as usize as RawPtr)
    }

    const fn into_asm(self) -> *mut c_void {
        self.0
    }
}

impl From<usize> for Arg {
    fn from(v: usize) -> Self {
        Self(v as RawPtr)
    }
}

impl From<c_int> for Arg {
    fn from(v: c_int) -> Self {
        Self(v as usize as RawPtr)
    }
}

impl<T> From<*mut T> for Arg {
    fn from(v: *mut T) -> Self {
        Self(v.cast())
    }
}

impl<T> From<*const T> for Arg {
    fn from(v: *const T) -> Self {
        Self(v.cast())
    }
}

type Errno = i64;

macro_rules! syscall {
    ($trap:expr, $arg1:expr) => {
        syscall3($trap, $arg1.into(), Arg::none(), Arg::none())
    };
    ($trap:expr, $arg1:expr, $arg2:expr, $arg3:expr) => {
        syscall3($trap, $arg1.into(), $arg2.into(), $arg3.into())
    };
}

unsafe fn syscall3(trap: i64, a1: Arg, a2: Arg, a3: Arg) -> Result<(i64, i64), Errno> {
    let r1;
    let r2;
    let ok: i64;
    asm!(
        "syscall",
        "setc r8b",
        "movzx {ok}, r8b",

        inlateout("rax") trap => r1,
        in("rdi") a1.into_asm(),
        in("rsi") a2.into_asm(),
        inlateout("rdx") a3.into_asm() => r2,
        ok = out(reg) ok,

        // FreeBSD clobbers these registers.
        out("r9") _,
        out("r10") _,

        // We clobber `dl`.
        out("r8b") _,

        options(nostack),
    );
    if ok != 0 {
        Ok((r1, r2))
    } else {
        Err(r1)
    }
}

#[no_mangle]
unsafe extern "C" fn exit(status: c_int) {
    let _ = syscall!(SYS_EXIT, status);
}

#[no_mangle]
unsafe extern "C" fn write(filedes: c_int, buf: *const c_void, nbyte: usize) -> isize {
    match syscall!(SYS_WRITE, filedes, buf, nbyte) {
        Ok((r0, _)) => r0 as isize,
        Err(_) => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    for i in 0..n {
        dst.add(i).write_volatile(src.read_volatile())
    }
    dst
}

#[no_mangle]
unsafe extern "C" fn memset(dst: *mut c_void, c: c_int, len: usize) -> *mut c_void {
    let ptr: *mut u8 = dst.cast();
    for i in 0..len {
        ptr.add(i).write_volatile(c as u8)
    }
    dst
}

#[no_mangle]
unsafe extern "C" fn _init_tls(_tls: *mut c_void) {}

#[no_mangle]
unsafe extern "C" fn atexit(_function: Option<extern "C" fn()>) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn main(_argc: c_int, _argv: *const *const c_char) -> c_int {
    let _ = writeln!(Stdout, "hello, world!");
    let auxv = AuxVec::from_static();
    let _ = writeln!(Stdout, "{auxv:#}");
    let _ = write!(Stdout, "\n");
    // SAFETY: FFI call, no invariants.
    unsafe { exit(33) }
    101
}

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
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
