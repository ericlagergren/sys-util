#![no_std]
#![no_main]
#![feature(lang_items)]
#![cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "solaris",
))]

use core::{
    arch::asm,
    ffi::{c_char, c_int, c_void},
    fmt::{self, Write},
};

use sys_auxv::AuxVec;

/*
TEXT	·Syscall(SB),NOSPLIT,$0-56
    CALL	runtime·entersyscall<ABIInternal>(SB)
    MOVQ	trap+0(FP), AX	// syscall entry
    MOVQ	a1+8(FP), DI
    MOVQ	a2+16(FP), SI
    MOVQ	a3+24(FP), DX
    SYSCALL
    JCC	ok
    MOVQ	$-1, r1+32(FP)	// r1
    MOVQ	$0, r2+40(FP)	// r2
    MOVQ	AX, err+48(FP)	// errno
    CALL	runtime·exitsyscall<ABIInternal>(SB)
    RET
ok:
    MOVQ	AX, r1+32(FP)	// r1
    MOVQ	DX, r2+40(FP)	// r2
    MOVQ	$0, err+48(FP)	// errno
    CALL	runtime·exitsyscall<ABIInternal>(SB)
    RET
*/
unsafe fn syscall(trap: i64, a1: i64, a2: i64, a3: i64) -> Result<(i64, i64), i64> {
    let r1;
    let r2;
    let ok: i64;
    asm!(
        "syscall",
        "setc r8b",
        "movzx r8b, {ok}",

        inlateout("rax") trap => r1,
        in("rdi") a1,
        in("rsi") a2,
        inlateout("rdx") a3 => r2,
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
unsafe extern "C" fn atexit(_function: Option<extern "C" fn()>) -> c_int {
    0
}

#[no_mangle]
unsafe extern "C" fn exit(_status: c_int) {}

#[no_mangle]
unsafe extern "C" fn write(filedes: c_int, buf: *const c_void, nbyte: usize) -> c_int {
    match syscall(4, filedes as i64, buf as i64, nbyte as i64) {
        Ok((r0, _)) => r0,
        Err(_) => -1,
    }
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
