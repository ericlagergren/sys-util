#![cfg(all(have_auxv, target_env = "gnu"))]

use core::ffi::c_char;

use super::sys::find_auxv;

pub fn auxv_via_argv() -> *const AuxVal {
    extern "C" {
        static _dl_argv: *const *const c_char;
    }
    let mut ptr = _dl_argv;
    while !(*ptr).is_null() {
        ptr = ptr.add(1);
    }
    find_auxv(ptr.add(1).cast())
}

#[cfg(feature = "rtld")]
pub mod rtld {
    use core::ffi::{c_char, c_int, c_ulong};

    pub(crate) unsafe fn auxv() -> *const AuxVal {
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
