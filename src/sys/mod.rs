#![cfg(have_auxv)]

mod generic;
mod glibc;
mod util;

use core::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use cfg_if::cfg_if;

use super::AuxVal;

/// Returns a pointer to the auxiliary vector.
pub fn auxv() -> *const AuxVal {
    let ptr = AUXV.load(Ordering::Relaxed);
    if !ptr.is_null() {
        ptr
    } else {
        find_auxv()
    }
}

fn find_auxv() -> *const AuxVal {
    cfg_if! {
        if #[cfg(feature = "glibc")] {
            let ptr = glibc::auxv().cast_mut();
        } else {
            let ptr = generic::auxv().cast_mut();
        }
    }
    if let Err(got) =
        AUXV.compare_exchange(ptr::null_mut(), ptr, Ordering::SeqCst, Ordering::Relaxed)
    {
        debug_assert_eq!(got, ptr);
    };
    ptr
}
static AUXV: AtomicPtr<AuxVal> = AtomicPtr::new(ptr::null_mut());
