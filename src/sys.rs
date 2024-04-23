#![cfg(have_auxv)]

use core::{
    ffi::c_char,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use super::auxv::AuxVal;

/// Returns a pointer to the auxiliary vector.
pub(crate) fn auxv() -> *const AuxVal {
    #[cfg(all(target_env = "gnu", feature = "rtld"))]
    {
        let ptr = super::glibc::rtld_auxv();
        if !ptr.is_null() {
            return ptr;
        }
    }

    let mut ptr = AUXV.load(Ordering::Relaxed);
    if ptr.is_null() {
        // SAFETY: `env` contains a valid process stack.
        ptr = unsafe { find_auxv(envp()) } as _;

        if let Err(got) =
            AUXV.compare_exchange(ptr::null_mut(), ptr, Ordering::SeqCst, Ordering::Relaxed)
        {
            debug_assert_eq!(got, ptr);
        };
    }
    ptr
}
static AUXV: AtomicPtr<AuxVal> = AtomicPtr::new(ptr::null_mut());

/// Finds the auxiliary vector using the process stack.
///
/// # Safety
///
/// The process stack must be correct.
unsafe fn find_auxv(envp: *const *const u8) -> *const AuxVal {
    let mut ptr = envp;
    while !(*ptr).is_null() {
        ptr = ptr.add(1);
    }
    ptr.add(1).cast()
}

fn envp() -> *const *const u8 {
    #[cfg(any(freebsdish, target_env = "gnu"))]
    if let Some(envp) = init_array::envp() {
        return envp;
    }
    // SAFETY: we just took the address of `environ`.
    let ptr = unsafe { *ENVIRON.load(Ordering::Relaxed) };
    ptr.cast()
}
extern "C" {
    static mut environ: *const *const c_char;
}
static ENVIRON: AtomicPtr<*const *const c_char> =
    // SAFETY: we just took the address of `environ`.
    AtomicPtr::new(unsafe { ptr::addr_of!(environ).cast_mut() });

#[cfg(any(freebsdish, target_env = "gnu"))]
mod init_array {
    use core::{
        ffi::c_int,
        ptr,
        sync::atomic::{AtomicBool, AtomicPtr, Ordering},
    };

    pub fn envp() -> Option<*const *const u8> {
        if INIT.load(Ordering::Relaxed) {
            Some(ENVP.load(Ordering::Relaxed))
        } else {
            None
        }
    }

    static INIT: AtomicBool = AtomicBool::new(false);
    static ENVP: AtomicPtr<*const u8> = AtomicPtr::new(ptr::null_mut());

    #[link_section = ".init_array.00099"]
    #[used]
    static ARGV_INIT_ARRAY: extern "C" fn(c_int, *const *const u8, *const *const u8) = init;

    extern "C" fn init(_argc: c_int, _argv: *const *const u8, envp: *const *const u8) {
        ENVP.store(envp.cast_mut(), Ordering::Relaxed);
        INIT.store(true, Ordering::Relaxed);
    }
}
