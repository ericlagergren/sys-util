use core::{
    ffi::c_char,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use super::{util::find_term, AuxVal};

/// Returns a pointer to the auxiliary vector.
pub(super) fn auxv() -> *const AuxVal {
    let ptr = envp();
    if !ptr.is_null() {
        // SAFETY: we've checked that `ptr` is non-null.
        unsafe { find_term(ptr) }.add(1).cast()
    } else {
        ptr::null()
    }
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
