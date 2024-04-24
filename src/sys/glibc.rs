#![cfg(feature = "glibc")]

use core::{ffi::c_char, ptr};

use super::{util::find_term, AuxVal};

/// Returns a pointer to the auxiliary vector.
pub(super) fn auxv() -> *const AuxVal {
    #[cfg(feature = "rtld")]
    {
        let ptr = rtld::auxv();
        if !ptr.is_null() {
            return ptr;
        }
    }

    let argv = argv();
    if argv.is_null() {
        return ptr::null();
    }
    // SAFETY: we've checked that `argv` is non-null.
    let envp = unsafe { find_term(argv).add(1) };
    if envp.is_null() {
        return ptr::null();
    }
    // SAFETY: we've checked that `ptr` is non-null.
    unsafe { find_term(envp).add(1) }.cast()
}

fn argv() -> *const *const u8 {
    extern "C" {
        static _dl_argv: *const *const c_char;
    }
    // SAFETY: the dynamic linker must have defined `_dl_argv`.
    unsafe { _dl_argv }
}

#[cfg(feature = "rtld")]
pub mod rtld {
    use core::ffi::{c_char, c_int, c_ulong};

    pub(crate) fn auxv() -> *const AuxVal {
        extern "C" {
            static _rtld_global_ro: RtldGlobal;
        }
        // SAFETY: fingers crossed that `RtldGlobal` is accurate,
        // etc.
        unsafe { _rtld_global_ro._dl_auxv }
    }

    mod v235 {
        #[derive(Debug)]
        #[repr(C)]
        struct RScopeElem {
            _r_list: *mut *mut (),
            _r_nlist: c_int,
        }
        #[derive(Debug)]
        #[repr(C)]
        struct RtldGlobal {
            _dl_debug_mask: c_int,
            _dl_osversion: c_uint,
            _dl_platform: *const c_char,
            _dl_platformlen: usize,
            _dl_pagesize: usize,
            _dl_minsigstacksize: usize,
            _dl_inhibit_cache: c_int,
            _dl_initial_searchlist: RScopeElem,
            _dl_clktck: c_int,
            _dl_verbose: c_int,
            _dl_debug_fd: c_int,
            _dl_lazy: c_int,
            _dl_bind_not: c_int,
            _dl_dynamic_weak: c_int,
            _dl_fpu_control: c_ulong,
            _dl_correct_cache_id: c_int,
            _dl_hwcap: u64,
            _dl_auxv: *const AuxVal,
        }
    }
}
