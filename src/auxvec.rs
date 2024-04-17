//! ELF auxiliary vector support.

use core::{
    ffi::c_int,
    fmt, mem, ptr, slice,
    sync::atomic::{AtomicIsize, AtomicPtr, Ordering},
};

use cfg_if::cfg_if;
use strum::{FromRepr, VariantArray, VariantNames};

macro_rules! const_assert {
    ($($tt:tt)*) => {
        const _: () = assert!($($tt)*);
    }
}

cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        /// An ELF word.
        pub type Word = u32;
    } else if #[cfg(target_pointer_width = "64")] {
        /// An ELF word.
        pub type Word = u64;
    } else {
        compile_error!("unknown pointer size");
    }
}

static AUXV: AtomicPtr<RawAuxVal> = AtomicPtr::new(ptr::null_mut());

/// The ELF auxiliary vector.
#[derive(Debug)]
#[repr(transparent)]
pub struct AuxVec([AuxVal]);

impl AuxVec {
    /// Returns the auxiliary vector for the current process.
    pub fn from_static() -> &'static Self {
        let result = AUXV.fetch_update(Ordering::SeqCst, Ordering::Relaxed, |old| {
            // SAFETY: `init` is called via the `.init_array`
            // constructor.
            let ptr = unsafe { find_auxv() } as _;
            debug_assert!(old.is_null() || ptr == old);
            Some(ptr)
        });
        let ptr = match result {
            Ok(ptr) | Err(ptr) => ptr,
        };
        // SAFETY: `ptr` came from `find_auxv`, which returns
        // a suitable pointer.
        unsafe { Self::from_ptr(ptr) }
    }

    /// # Safety
    ///
    /// - `ptr` must be non-null and point to a valid auxiliary
    /// vector.
    unsafe fn from_ptr(ptr: *const RawAuxVal) -> &'static Self {
        debug_assert!(!ptr.is_null());

        let mut len = 0;
        loop {
            let value = ptr.add(len);
            if (*value).key == Type::Null.to_word() {
                break;
            }
            len += 1;
        }
        Self::from_raw_parts(ptr.cast(), len)
    }

    /// # Safety
    ///
    /// Same as [`slice::from_raw_parts`].
    unsafe fn from_raw_parts(ptr: *const AuxVal, len: usize) -> &'static Self {
        // SAFETY: see the doc comment.
        let v = unsafe { slice::from_raw_parts(ptr, len) };
        // SAFETY: `[AuxVal]` and `Self` have the same memory
        // layout.
        unsafe { &*(v as *const [AuxVal] as *const AuxVec) }
    }

    /// Returns an iterator over the auxiliary vector.
    pub fn iter(&self) -> impl Iterator<Item = &AuxVal> {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a AuxVec {
    type Item = &'a AuxVal;
    type IntoIter = slice::Iter<'a, AuxVal>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Display for AuxVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for value in self {
            write!(f, "{value}")?;
        }
        Ok(())
    }
}

/// An auxiliary vector key-value pair.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AuxVal {
    /// The key.
    pub key: Type,
    /// The value.
    pub val: Word,
}

impl fmt::Display for AuxVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.val)
    }
}

/// The type of an [`AuxVal`].
#[derive(
    Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, FromRepr, VariantArray, VariantNames,
)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
#[allow(missing_docs)] // TODO
pub enum Type {
    /// `AT_NULL`
    Null = 0,
    /// `AT_IGNORE`
    Ignore = 1,
    /// `AT_EXECFD`
    ExecFd = 2,
    /// `AT_PHDR`.
    Phdr = 3,
    /// `AT_PHENT`.
    Phent = 4,
    /// `AT_PHNUM`.
    Phnum = 5,
    /// `AT_PAGESZ`.
    PageSize = 6,
    Base = 7,
    Flags = 8,
    Entry = 9,
    NotElf = 10,
    Uid = 11,
    Euid = 12,
    Gid = 13,
    Egid = 14,
    ClockTick = 17,
    Platform = 15,
    Hwcap = 16,
    Fpucw = 18,
    DataCacheBlockSize = 19,
    InstCacheBlockSize = 20,
    UnifiedCacheBlockSize = 21,
    IgnorePpc = 22,
    Secure = 23,
    BasePlatfomr = 24,
    Random = 25,
    Hwcap2 = 26,
    RseqFeatureSize = 27,
    RseqAlign = 28,
    HwCap3 = 29,
    HwCap4 = 30,
    ExecFn = 31,
    Sysinfo = 32,
    SysinfoEhdr = 33,
    L1InstCacheShape = 34,
    L1DataCacheChape = 35,
    L2CacheShape = 36,
    L3CacheShape = 37,
    L1InstCacheSize = 40,
    L1InstCacheGeometry = 41,
    L1DCacheSize = 42,
    L1DCacheGeometry = 43,
    L2CacheSize = 44,
    L2CacheGeometry = 45,
    L3CacheSize = 46,
    L3CacheGeometry = 47,
    MinSigStackSize = 51,
}
const_assert!(mem::size_of::<Type>() == mem::size_of::<Word>());

impl Type {
    /// Converts a `Word` to a `Type`.
    pub const fn from_u64(v: Word) -> Option<Self> {
        Self::from_repr(v)
    }

    /// Converts the `Type` to a `Word`.
    pub const fn to_word(self) -> Word {
        self as Word
    }

    /// Returns the string encoding of the type.
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::Null => "AT_NULL",
            Self::Ignore => "AT_IGNORE",
            Self::ExecFd => "AT_EXECFD",
            Self::Phdr => "AT_PHDR",
            Self::Phent => "AT_PHENT",
            Self::Phnum => "AT_PHNUM",
            Self::PageSize => "AT_PAGESZ",
            Self::Base => "AT_BASE",
            Self::Flags => "AT_FLAGS",
            Self::Entry => "AT_ENTRY",
            Self::NotElf => "AT_NOTELF",
            Self::Uid => "AT_UID",
            Self::Euid => "AT_EUID",
            Self::Gid => "AT_GID",
            Self::Egid => "AT_EGID",
            Self::ClockTick => "AT_CLKTCK",
            Self::Platform => "AT_PLATFORM",
            Self::Hwcap => "AT_HWCAP",
            Self::Fpucw => "AT_FPUCW",
            Self::DataCacheBlockSize => "AT_DCACHEBSIZE",
            Self::InstCacheBlockSize => "AT_ICACHEBSIZE",
            Self::UnifiedCacheBlockSize => "AT_UCACHEBSIZE",
            Self::IgnorePpc => "AT_IGNOREPPC",
            Self::Secure => "AT_SECURE",
            Self::BasePlatfomr => "AT_BASE_PLATFORM",
            Self::Random => "AT_RANDOM",
            Self::Hwcap2 => "AT_HWCAP2",
            Self::RseqFeatureSize => "AT_RSEQ_FEATURE_SIZE",
            Self::RseqAlign => "AT_RSEQ_ALIGN",
            Self::HwCap3 => "AT_HWCAP3",
            Self::HwCap4 => "AT_HWCAP4",
            Self::ExecFn => "AT_EXECFN",
            Self::Sysinfo => "AT_SYSINFO",
            Self::SysinfoEhdr => "AT_SYSINFO_EHDR",
            Self::L1InstCacheShape => "AT_L1I_CACHESHAPE",
            Self::L1DataCacheChape => "AT_L1D_CACHESHAPE",
            Self::L2CacheShape => "AT_L2_CACHESHAPE",
            Self::L3CacheShape => "AT_L3_CACHESHAPE",
            Self::L1InstCacheSize => "AT_L1I_CACHESIZE",
            Self::L1InstCacheGeometry => "AT_L1I_CACHEGEOMETRY",
            Self::L1DCacheSize => "AT_L1D_CACHESIZE",
            Self::L1DCacheGeometry => "AT_L1D_CACHEGEOMETRY",
            Self::L2CacheSize => "AT_L2_CACHESIZE",
            Self::L2CacheGeometry => "AT_L2_CACHEGEOMETRY",
            Self::L3CacheSize => "AT_L3_CACHESIZE",
            Self::L3CacheGeometry => "AT_L3_CACHEGEOMETRY",
            Self::MinSigStackSize => "AT_MINSIGSTKSZ",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

/// # Safety
///
/// `init` must be called first.
unsafe fn find_auxv() -> *const RawAuxVal {
    let argc = ARGC.load(Ordering::Relaxed);
    let argv = ARGV.load(Ordering::Relaxed);
    let envp = ENVP.load(Ordering::Relaxed);

    #[cfg(test)]
    {
        println!("argc = {argc}");
        println!("argv = {argv:p}");
        println!("envp = {envp:p}");
    }

    for i in 0..argc {
        let ptr = *argv.offset(i);
        if ptr.is_null() {
            break;
        }
        #[cfg(test)]
        {
            #[allow(clippy::unwrap_used)]
            let arg = ::core::ffi::CStr::from_ptr(ptr.cast()).to_str().unwrap();
            println!("#{i}: {arg}");
        }
    }

    let mut ptr = envp; // argv.offset(argc + 1);
    while !(*ptr).is_null() {
        ptr = ptr.add(1);
    }
    ptr.add(1).cast()
}

#[repr(C)]
struct RawAuxVal {
    key: Word,
    val: Word,
}

static ARGC: AtomicIsize = AtomicIsize::new(0);
static ARGV: AtomicPtr<*const u8> = AtomicPtr::new(ptr::null_mut());
static ENVP: AtomicPtr<*const u8> = AtomicPtr::new(ptr::null_mut());

#[link_section = ".init_array.00099"]
#[used]
static ARGV_INIT_ARRAY: extern "C" fn(c_int, *const *const u8, *const *const u8) = init;

extern "C" fn init(argc: c_int, argv: *const *const u8, envp: *const *const u8) {
    ARGC.store(argc as isize, Ordering::Relaxed);
    ARGV.store(argv.cast_mut(), Ordering::Relaxed);
    ENVP.store(envp.cast_mut(), Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let v = AuxVec::from_static();
        println!("{v}");
    }
}
