//! ELF auxiliary vector support.

// TODO: /proc/self/auxv

use core::{
    fmt::{self, Display},
    slice,
};

use cfg_if::cfg_if;

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

/// See libc's `getauxval`.
#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "solaris",
))]
#[cfg_attr(
    docs,
    doc(cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "solaris",
    )))
)]
pub fn getauxval(key: Type) -> Option<Word> {
    // TODO
    // use core::sync::atomic::{AtomicU64, Ordering};
    // const HWCAPS: AtomicU64 = AtomicU64::new(0);

    let aux = AuxVec::from_static();
    for v in aux {
        println!("v={v:#}");
        if v.key == key {
            return Some(v.val);
        }
    }
    None
}

/// The ELF auxiliary vector.
#[derive(Debug)]
#[repr(transparent)]
pub struct AuxVec([AuxVal]);

impl AuxVec {
    /// Returns the auxiliary vector from the process stack.
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "solaris",
    ))]
    #[cfg_attr(
        docs,
        doc(cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "illumos",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "solaris",
        )))
    )]
    pub fn from_static() -> &'static Self {
        // SAFETY: `ptr` came from `rt::auxv`, which returns
        // a suitable pointer.
        unsafe { Self::from_ptr(rt::auxv()) }
    }

    /// Creates an `AuxVec` from a raw pointer to an auxiliary
    /// vector.
    ///
    /// # Safety
    ///
    /// - `ptr` must be non-null and point to a valid auxiliary
    /// vector.
    pub unsafe fn from_ptr(ptr: *const AuxVal) -> &'static Self {
        debug_assert!(!ptr.is_null());

        let mut len = 0;
        loop {
            let value = ptr.add(len);
            if (*value).key == Type::AT_NULL {
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

impl Display for AuxVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for value in self {
            if f.alternate() {
                writeln!(f, "{value:#}")?;
            } else {
                writeln!(f, "{value}")?;
            }
        }
        Ok(())
    }
}

/// An auxiliary vector key-value pair.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct AuxVal {
    /// The key.
    pub key: Type,
    /// The value.
    pub val: Word,
}

impl AuxVal {
    #[cfg(target_os = "linux")]
    fn write_val_alt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.key {
            Type::AT_PHENT
            | Type::AT_PHNUM
            | Type::AT_PAGESZ
            | Type::AT_UID
            | Type::AT_EUID
            | Type::AT_GID
            | Type::AT_EGID
            | Type::AT_CLKTCK
            | Type::AT_SECURE
            | Type::AT_NOTELF
            | Type::AT_MINSIGSTKSZ => self.val.fmt(f),
            Type::AT_EXECFN | Type::AT_PLATFORM => {
                let ptr = self.val as *const i8;
                if !ptr.is_null() {
                    // SAFETY: we know that `ptr` is non-null,
                    // but we have to trust that it is
                    // null-terminated.
                    let s = unsafe { core::ffi::CStr::from_ptr(ptr) };
                    s.to_str().unwrap_or("???").fmt(f)
                } else {
                    "???".fmt(f)
                }
            }
            _ => self.write_val_hex(f),
        }
    }

    #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
    fn write_val_alt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.key {
            Type::AT_PHENT
            | Type::AT_PHNUM
            | Type::AT_PAGESZ
            | Type::AT_UID
            | Type::AT_EUID
            | Type::AT_GID
            | Type::AT_EGID
            | Type::AT_STACKPROT => self.val.fmt(f),
            _ => self.write_val_hex(f),
        }
    }

    #[cfg(not(any(target_os = "dragonfly", target_os = "freebsd", target_os = "linux")))]
    fn write_val_alt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.key {
            Type::AT_PHENT
            | Type::AT_PHNUM
            | Type::AT_PAGESZ
            | Type::AT_UID
            | Type::AT_EUID
            | Type::AT_GID
            | Type::AT_EGID => self.val.fmt(f),
            _ => self.write_val_hex(f),
        }
    }

    fn write_val_hex(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.val)
    }
}

impl Display for AuxVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<20}: ", self.key)?;
        if f.alternate() {
            self.write_val_alt(f)
        } else {
            self.write_val_hex(f)
        }
    }
}

/// The type of an [`AuxVal`].
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Type(Word);

impl Type {
    /// `AT_NULL`.
    pub const AT_NULL: Self = Self(0);
    /// `AT_IGNORE`.
    pub const AT_IGNORE: Self = Self(1);
    /// `AT_EXECFD`.
    pub const AT_EXECFD: Self = Self(2);
    /// `AT_PHDR`.
    pub const AT_PHDR: Self = Self(3);
    /// `AT_PHENT`.
    pub const AT_PHENT: Self = Self(4);
    /// `AT_PHNUM`.
    pub const AT_PHNUM: Self = Self(5);
    /// `AT_PAGESZ`.
    pub const AT_PAGESZ: Self = Self(6);
    /// `AT_BASE`.
    pub const AT_BASE: Self = Self(7);
    /// `AT_FLAGS`.
    pub const AT_FLAGS: Self = Self(8);
    /// `AT_ENTRY`.
    pub const AT_ENTRY: Self = Self(9);
    /// `AT_NOTELF`.
    pub const AT_NOTELF: Self = Self(10);
    /// `AT_UID`.
    pub const AT_UID: Self = Self(11);
    /// `AT_EUID`.
    pub const AT_EUID: Self = Self(12);
    /// `AT_GID`.
    pub const AT_GID: Self = Self(13);
    /// `AT_EGID`.
    pub const AT_EGID: Self = Self(14);
    /// `AT_HWCAP2`.
    pub const AT_HWCAP2: Self = Self(26);
}

#[cfg(target_os = "linux")]
#[cfg_attr(docs, doc(cfg(target_os = "linux")))]
impl Type {
    /// `AT_PLATFORM`.
    pub const AT_PLATFORM: Self = Self(15);
    /// `AT_HWCAP`.
    pub const AT_HWCAP: Self = Self(16);
    /// `AT_CLKTCK`.
    pub const AT_CLKTCK: Self = Self(17);

    /// `AT_SECURE`.
    pub const AT_SECURE: Self = Self(23);
    /// `AT_BASE_PLATFORM`.
    pub const AT_BASE_PLATFORM: Self = Self(24);
    /// `AT_RANDOM`.
    pub const AT_RANDOM: Self = Self(25);
    /// `AT_RSEQ_FEATURE_SIZE`.
    pub const AT_RSEQ_FEATURE_SIZE: Self = Self(27);
    /// `AT_RSEQ_ALIGN`.
    pub const AT_RSEQ_ALIGN: Self = Self(28);
    /// `AT_HWCAP3`.
    pub const AT_HWCAP3: Self = Self(29);
    /// `AT_HWCAP4`.
    pub const AT_HWCAP4: Self = Self(30);
    /// `AT_EXECFN`.
    pub const AT_EXECFN: Self = Self(31);
    /// `AT_SYSINFO`.
    pub const AT_SYSINFO: Self = Self(32);
    /// `AT_SYSINFO_EHDR`.
    pub const AT_SYSINFO_EHDR: Self = Self(33);
    /// `AT_L1I_CACHESHAPE`.
    pub const AT_L1I_CACHESHAPE: Self = Self(34);
    /// `AT_L1D_CACHESHAPE`.
    pub const AT_L1D_CACHESHAPE: Self = Self(35);
    /// `AT_L2_CACHESHAPE`.
    pub const AT_L2_CACHESHAPE: Self = Self(36);
    /// `AT_L3_CACHESHAPE`.
    pub const AT_L3_CACHESHAPE: Self = Self(37);
    /// `AT_L1I_CACHESIZE`.
    pub const AT_L1I_CACHESIZE: Self = Self(40);
    /// `AT_L1I_CACHEGEOMETRY`.
    pub const AT_L1I_CACHEGEOMETRY: Self = Self(41);
    /// `AT_L1D_CACHESIZE`.
    pub const AT_L1D_CACHESIZE: Self = Self(42);
    /// `AT_L1D_CACHEGEOMETRY`.
    pub const AT_L1D_CACHEGEOMETRY: Self = Self(43);
    /// `AT_L2_CACHESIZE`.
    pub const AT_L2_CACHESIZE: Self = Self(44);
    /// `AT_L2_CACHEGEOMETRY`.
    pub const AT_L2_CACHEGEOMETRY: Self = Self(45);
    /// `AT_L3_CACHESIZE`.
    pub const AT_L3_CACHESIZE: Self = Self(46);
    /// `AT_L3_CACHEGEOMETRY`.
    pub const AT_L3_CACHEGEOMETRY: Self = Self(47);
    /// `AT_MINSIGSTKSZ`.
    pub const AT_MINSIGSTKSZ: Self = Self(51);
}

#[cfg(all(target_os = "linux", target_arch = "powerpc"))]
#[cfg_attr(docs, doc(cfg(all(target_os = "linux", target_arch = "powerpc"))))]
impl Type {
    /// `AT_FPUCW`.
    pub const AT_FPUCW: Self = Self(18);
    /// `AT_DCACHEBSIZE`.
    pub const AT_DCACHEBSIZE: Self = Self(19);
    /// `AT_ICACHEBSIZE`.
    pub const AT_ICACHEBSIZE: Self = Self(20);
    /// `AT_UCACHEBSIZE`.
    pub const AT_UCACHEBSIZE: Self = Self(21);
    /// `AT_IGNOREPPC`.
    pub const AT_IGNOREPPC: Self = Self(22);
}

#[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
#[cfg_attr(docs, doc(cfg(any(target_os = "dragonfly", target_os = "freebsd"))))]
impl Type {
    /// `AT_EXECPATH`.
    pub const AT_EXECPATH: Self = Self(15);
    /// `AT_CANARY`.
    pub const AT_CANARY: Self = Self(16);
    /// `AT_CANARYLEN`.
    pub const AT_CANARYLEN: Self = Self(17);
    /// `AT_OSRELDATE`.
    pub const AT_OSRELDATE: Self = Self(18);
    /// `AT_NCPUS`.
    pub const AT_NCPUS: Self = Self(19);
    /// `AT_PAGESIZES`.
    pub const AT_PAGESIZES: Self = Self(20);
    /// `AT_PAGESIZESLEN`.
    pub const AT_PAGESIZESLEN: Self = Self(21);
    /// `AT_TIMEKEEP`.
    pub const AT_TIMEKEEP: Self = Self(22);
    /// `AT_STACKPROT`.
    pub const AT_STACKPROT: Self = Self(23);
    /// `AT_EHDRFLAGS`.
    pub const AT_EHDRFLAGS: Self = Self(24);
    /// `AT_HWCAP`.
    pub const AT_HWCAP: Self = Self(25);
    /// `AT_BSDFLAGS`.
    pub const AT_BSDFLAGS: Self = Self(27);
    /// `AT_ARGC`.
    pub const AT_ARGC: Self = Self(28);
    /// `AT_ARGV`.
    pub const AT_ARGV: Self = Self(29);
    /// `AT_ENVC`.
    pub const AT_ENVC: Self = Self(30);
    /// `AT_ENVV`.
    pub const AT_ENVV: Self = Self(31);
    /// `AT_PS_STRINGS`.
    pub const AT_PS_STRINGS: Self = Self(32);
    /// `AT_FXRNG`.
    pub const AT_FXRNG: Self = Self(33);
    /// `AT_KPRELOAD`.
    pub const AT_KPRELOAD: Self = Self(34);
    /// `AT_USRSTACKBASE`.
    pub const AT_USRSTACKBASE: Self = Self(35);
    /// `AT_USRSTACKLIM`.
    pub const AT_USRSTACKLIM: Self = Self(36);
    /// `AT_COUNT`.
    pub const AT_COUNT: Self = Self(37);
}

impl Type {
    /// Converts the `Type` to a string.
    pub const fn to_str(self) -> Option<&'static str> {
        if let Some(s) = self.to_str_base() {
            return Some(s);
        };
        self.to_str_os()
    }

    const fn to_str_base(self) -> Option<&'static str> {
        let s = match self {
            Self::AT_NULL => "AT_NULL",
            Self::AT_IGNORE => "AT_IGNORE",
            Self::AT_EXECFD => "AT_EXECFD",
            Self::AT_PHDR => "AT_PHDR",
            Self::AT_PHENT => "AT_PHENT",
            Self::AT_PHNUM => "AT_PHNUM",
            Self::AT_PAGESZ => "AT_PAGESZ",
            Self::AT_BASE => "AT_BASE",
            Self::AT_FLAGS => "AT_FLAGS",
            Self::AT_ENTRY => "AT_ENTRY",
            Self::AT_NOTELF => "AT_NOTELF",
            Self::AT_UID => "AT_UID",
            Self::AT_EUID => "AT_EUID",
            Self::AT_GID => "AT_GID",
            Self::AT_EGID => "AT_EGID",
            Self::AT_HWCAP2 => "AT_HWCAP2",
            _ => return None,
        };
        Some(s)
    }

    #[cfg(target_os = "linux")]
    const fn to_str_os(self) -> Option<&'static str> {
        let s = match self {
            Self::AT_PLATFORM => "AT_PLATFORM",
            Self::AT_HWCAP => "AT_HWCAP",
            Self::AT_CLKTCK => "AT_CLKTCK",
            #[cfg(feature = "esoteric")]
            Self::AT_FPUCW => "AT_FPUCW",
            #[cfg(feature = "esoteric")]
            Self::AT_DCACHEBSIZE => "AT_DCACHEBSIZE",
            #[cfg(feature = "esoteric")]
            Self::AT_ICACHEBSIZE => "AT_ICACHEBSIZE",
            #[cfg(feature = "esoteric")]
            Self::AT_UCACHEBSIZE => "AT_UCACHEBSIZE",
            #[cfg(feature = "esoteric")]
            Self::AT_IGNOREPPC => "AT_IGNOREPPC",
            Self::AT_SECURE => "AT_SECURE",
            Self::AT_BASE_PLATFORM => "AT_BASE_PLATFORM",
            Self::AT_RANDOM => "AT_RANDOM",
            Self::AT_RSEQ_FEATURE_SIZE => "AT_RSEQ_FEATURE_SIZE",
            Self::AT_RSEQ_ALIGN => "AT_RSEQ_ALIGN",
            Self::AT_HWCAP3 => "AT_HWCAP3",
            Self::AT_HWCAP4 => "AT_HWCAP4",
            Self::AT_EXECFN => "AT_EXECFN",
            Self::AT_SYSINFO => "AT_SYSINFO",
            Self::AT_SYSINFO_EHDR => "AT_SYSINFO_EHDR",
            Self::AT_L1I_CACHESHAPE => "AT_L1I_CACHESHAPE",
            Self::AT_L1D_CACHESHAPE => "AT_L1D_CACHESHAPE",
            Self::AT_L2_CACHESHAPE => "AT_L2_CACHESHAPE",
            Self::AT_L3_CACHESHAPE => "AT_L3_CACHESHAPE",
            Self::AT_L1I_CACHESIZE => "AT_L1I_CACHESIZE",
            Self::AT_L1I_CACHEGEOMETRY => "AT_L1I_CACHEGEOMETRY",
            Self::AT_L1D_CACHESIZE => "AT_L1D_CACHESIZE",
            Self::AT_L1D_CACHEGEOMETRY => "AT_L1D_CACHEGEOMETRY",
            Self::AT_L2_CACHESIZE => "AT_L2_CACHESIZE",
            Self::AT_L2_CACHEGEOMETRY => "AT_L2_CACHEGEOMETRY",
            Self::AT_L3_CACHESIZE => "AT_L3_CACHESIZE",
            Self::AT_L3_CACHEGEOMETRY => "AT_L3_CACHEGEOMETRY",
            Self::AT_MINSIGSTKSZ => "AT_MINSIGSTKSZ",
            _ => return None,
        };
        Some(s)
    }

    #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
    const fn to_str_os(self) -> Option<&'static str> {
        let s = match self {
            Self::AT_EXECPATH => "AT_EXECPATH",
            Self::AT_CANARY => "AT_CANARY",
            Self::AT_CANARYLEN => "AT_CANARYLEN",
            Self::AT_OSRELDATE => "AT_OSRELDATE",
            Self::AT_NCPUS => "AT_NCPUS",
            Self::AT_PAGESIZES => "AT_PAGESIZES",
            Self::AT_PAGESIZESLEN => "AT_PAGESIZESLEN",
            Self::AT_TIMEKEEP => "AT_TIMEKEEP",
            Self::AT_STACKPROT => "AT_STACKPROT",
            Self::AT_EHDRFLAGS => "AT_EHDRFLAGS",
            Self::AT_HWCAP => "AT_HWCAP",
            Self::AT_BSDFLAGS => "AT_BSDFLAGS",
            Self::AT_ARGC => "AT_ARGC",
            Self::AT_ARGV => "AT_ARGV",
            Self::AT_ENVC => "AT_ENVC",
            Self::AT_ENVV => "AT_ENVV",
            Self::AT_PS_STRINGS => "AT_PS_STRINGS",
            Self::AT_FXRNG => "AT_FXRNG",
            Self::AT_KPRELOAD => "AT_KPRELOAD",
            Self::AT_USRSTACKBASE => "AT_USRSTACKBASE",
            Self::AT_USRSTACKLIM => "AT_USRSTACKLIM",
            Self::AT_COUNT => "AT_COUNT",
            _ => return None,
        };
        Some(s)
    }

    #[cfg(not(any(target_os = "dragonfly", target_os = "freebsd", target_os = "linux")))]
    const fn to_str_os(self) -> Option<&'static str> {
        None
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.to_str() {
            s.fmt(f)
        } else {
            write!(f, "AT_??? ({})", self.0)
        }
    }
}

impl PartialEq<Word> for Type {
    fn eq(&self, other: &Word) -> bool {
        PartialEq::eq(&self.0, other)
    }
}

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "linux",
    target_os = "netbsd",
    target_os = "solaris",
))]
mod rt {
    use core::{
        ffi::c_char,
        ptr,
        sync::atomic::{AtomicPtr, Ordering},
    };

    use super::AuxVal;

    /// Returns a pointer to the auxiliary vector.
    pub fn auxv() -> *const AuxVal {
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
        #[cfg(any(target_os = "freebsd", target_env = "gnu"))]
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

    #[cfg(any(target_os = "freebsd", target_env = "gnu"))]
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
}

#[cfg(all(
    test,
    any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "solaris",
    )
))]
mod tests {
    use core::ffi::{c_int, c_ulong};

    use super::*;

    const BASE_TYPES: [(Type, Word); 16] = [
        (Type::AT_NULL, libc::AT_NULL),
        (Type::AT_IGNORE, libc::AT_IGNORE),
        (Type::AT_EXECFD, libc::AT_EXECFD),
        (Type::AT_PHDR, libc::AT_PHDR),
        (Type::AT_PHENT, libc::AT_PHENT),
        (Type::AT_PHNUM, libc::AT_PHNUM),
        (Type::AT_PAGESZ, libc::AT_PAGESZ),
        (Type::AT_BASE, libc::AT_BASE),
        (Type::AT_FLAGS, libc::AT_FLAGS),
        (Type::AT_ENTRY, libc::AT_ENTRY),
        (Type::AT_NOTELF, libc::AT_NOTELF),
        (Type::AT_UID, libc::AT_UID),
        (Type::AT_EUID, libc::AT_EUID),
        (Type::AT_GID, libc::AT_GID),
        (Type::AT_EGID, libc::AT_EGID),
        (Type::AT_HWCAP2, libc::AT_HWCAP2),
    ];

    // Commented out types are currently not included in `libc`.
    #[cfg(target_os = "freebsd")]
    const OS_TYPES: [(Type, Word); 8] = [
        (Type::AT_EXECPATH, libc::AT_EXECPATH as Word),
        (Type::AT_CANARY, libc::AT_CANARY as Word),
        // (Type::AT_CANARYLEN, libc::AT_CANARYLEN as Word),
        (Type::AT_OSRELDATE, libc::AT_OSRELDATE as Word),
        (Type::AT_NCPUS, libc::AT_NCPUS as Word),
        // (Type::AT_PAGESIZES, libc::AT_PAGESIZES as Word),
        // (Type::AT_PAGESIZESLEN, libc::AT_PAGESIZESLEN as Word),
        (Type::AT_TIMEKEEP, libc::AT_TIMEKEEP as Word),
        // (Type::AT_STACKPROT, libc::AT_STACKPROT as Word),
        // (Type::AT_EHDRFLAGS, libc::AT_EHDRFLAGS as Word),
        (Type::AT_HWCAP, libc::AT_HWCAP as Word),
        // (Type::AT_BSDFLAGS, libc::AT_BSDFLAGS as Word),
        // (Type::AT_ARGC, libc::AT_ARGC as Word),
        // (Type::AT_ARGV, libc::AT_ARGV as Word),
        // (Type::AT_ENVC, libc::AT_ENVC as Word),
        // (Type::AT_ENVV, libc::AT_ENVV as Word),
        // (Type::AT_PS_STRINGS, libc::AT_PS_STRINGS as Word),
        // (Type::AT_FXRNG, libc::AT_FXRNG as Word),
        // (Type::AT_KPRELOAD, libc::AT_KPRELOAD as Word),
        (Type::AT_USRSTACKBASE, libc::AT_USRSTACKBASE as Word),
        (Type::AT_USRSTACKLIM, libc::AT_USRSTACKLIM as Word),
        // (Type::AT_COUNT, libc::AT_COUNT as Word),
    ];

    // Commented out types are currently not included in `libc`.
    #[cfg(target_os = "linux")]
    const OS_TYPES: [(Type, Word); 8] = [
        (Type::AT_PLATFORM, libc::AT_PLATFORM),
        (Type::AT_HWCAP, libc::AT_HWCAP),
        (Type::AT_CLKTCK, libc::AT_CLKTCK),
        // (Type::AT_FPUCW, libc::AT_FPUCW),
        // (Type::AT_DCACHEBSIZE, libc::AT_DCACHEBSIZE),
        // (Type::AT_ICACHEBSIZE, libc::AT_ICACHEBSIZE),
        // (Type::AT_UCACHEBSIZE, libc::AT_UCACHEBSIZE),
        // (Type::AT_IGNOREPPC, libc::AT_IGNOREPPC),
        (Type::AT_SECURE, libc::AT_SECURE),
        (Type::AT_BASE_PLATFORM, libc::AT_BASE_PLATFORM),
        (Type::AT_RANDOM, libc::AT_RANDOM),
        // (Type::AT_RSEQ_FEATURE_SIZE, libc::AT_RSEQ_FEATURE_SIZE),
        // (Type::AT_RSEQ_ALIGN, libc::AT_RSEQ_ALIGN),
        // (Type::AT_HWCAP3, libc::AT_HWCAP3),
        // (Type::AT_HWCAP4, libc::AT_HWCAP4),
        (Type::AT_EXECFN, libc::AT_EXECFN),
        // (Type::AT_SYSINFO, libc::AT_SYSINFO),
        (Type::AT_SYSINFO_EHDR, libc::AT_SYSINFO_EHDR),
        // (Type::AT_L1I_CACHESHAPE, libc::AT_L1I_CACHESHAPE),
        // (Type::AT_L1D_CACHESHAPE, libc::AT_L1D_CACHESHAPE),
        // (Type::AT_L2_CACHESHAPE, libc::AT_L2_CACHESHAPE),
        // (Type::AT_L3_CACHESHAPE, libc::AT_L3_CACHESHAPE),
        // (Type::AT_L1I_CACHESIZE, libc::AT_L1I_CACHESIZE),
        // (Type::AT_L1I_CACHEGEOMETRY, libc::AT_L1I_CACHEGEOMETRY),
        // (Type::AT_L1D_CACHESIZE, libc::AT_L1D_CACHESIZE),
        // (Type::AT_L1D_CACHEGEOMETRY, libc::AT_L1D_CACHEGEOMETRY),
        // (Type::AT_L2_CACHESIZE, libc::AT_L2_CACHESIZE),
        // (Type::AT_L2_CACHEGEOMETRY, libc::AT_L2_CACHEGEOMETRY),
        // (Type::AT_L3_CACHESIZE, libc::AT_L3_CACHESIZE),
        // (Type::AT_L3_CACHEGEOMETRY, libc::AT_L3_CACHEGEOMETRY),
        // (Type::AT_MINSIGSTKSZ, libc::AT_MINSIGSTKSZ),
    ];

    #[cfg(target_os = "linux")]
    fn sys_getauxval(type_: c_ulong) -> Option<c_ulong> {
        // SAFETY: FFI call, no invariants.
        let value = unsafe { libc::getauxval(type_) };
        if value == 0 {
            // SAFETY: FFI call, no invariants.
            let errno = unsafe { *libc::__errno_location() };
            if errno == libc::ENOENT {
                return None;
            }
        }
        Some(value)
    }

    #[cfg(target_os = "freebsd")]
    fn sys_getauxval(type_: c_int) -> Option<c_ulong> {
        use core::{mem, ptr};

        let mut out: c_ulong = 0;
        // SAFETY: FFI call, no invariants.
        let ret = unsafe {
            libc::elf_aux_info(
                type_,
                ptr::addr_of_mut!(out) as _,
                mem::size_of_val(&out) as c_int,
            )
        };
        if ret != 0 {
            None
        } else {
            Some(out)
        }
    }

    #[test]
    fn test_libc_compat() {
        let v = AuxVec::from_static();
        println!("{v:#}");

        for (got, want) in BASE_TYPES.into_iter().chain(OS_TYPES) {
            assert_eq!(got.0, want as Word);

            let got = getauxval(Type::AT_HWCAP);
            let want = sys_getauxval(libc::AT_HWCAP);
            println!(" got = {got:?}");
            println!("want = {want:?}");
            assert_eq!(got, want);
        }
    }
}
