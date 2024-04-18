//! ELF auxiliary vector support.

use core::{fmt, slice};

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

impl fmt::Display for AuxVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for value in self {
            writeln!(f, "{value}")?;
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

impl fmt::Display for AuxVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.val)
    }
}

/// The type of an [`AuxVal`].
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Type(Word);

impl Type {
    const AT_NULL: Self = Self(0);
    const AT_IGNORE: Self = Self(1);
    const AT_EXECFD: Self = Self(2);
    const AT_PHDR: Self = Self(3);
    const AT_PHENT: Self = Self(4);
    const AT_PHNUM: Self = Self(5);
    const AT_PAGESZ: Self = Self(6);
    const AT_BASE: Self = Self(7);
    const AT_FLAGS: Self = Self(8);
    const AT_ENTRY: Self = Self(9);
    const AT_NOTELF: Self = Self(10);
    const AT_UID: Self = Self(11);
    const AT_EUID: Self = Self(12);
    const AT_GID: Self = Self(13);
    const AT_EGID: Self = Self(14);

    const AT_HWCAP2: Self = Self(26);
}

#[cfg(target_os = "linux")]
impl Type {
    const AT_CLKTCK: Self = Self(17);
    const AT_PLATFORM: Self = Self(15);
    const AT_HWCAP: Self = Self(16);
    const AT_FPUCW: Self = Self(18);
    const AT_DCACHEBSIZE: Self = Self(19);
    const AT_ICACHEBSIZE: Self = Self(20);
    const AT_UCACHEBSIZE: Self = Self(21);
    const AT_IGNOREPPC: Self = Self(22);
    const AT_SECURE: Self = Self(23);
    const AT_BASE_PLATFORM: Self = Self(24);
    const AT_RANDOM: Self = Self(25);
    const AT_RSEQ_FEATURE_SIZE: Self = Self(27);
    const AT_RSEQ_ALIGN: Self = Self(28);
    const AT_HWCAP3: Self = Self(29);
    const AT_HWCAP4: Self = Self(30);
    const AT_EXECFN: Self = Self(31);
    const AT_SYSINFO: Self = Self(32);
    const AT_SYSINFO_EHDR: Self = Self(33);
    const AT_L1I_CACHESHAPE: Self = Self(34);
    const AT_L1D_CACHESHAPE: Self = Self(35);
    const AT_L2_CACHESHAPE: Self = Self(36);
    const AT_L3_CACHESHAPE: Self = Self(37);
    const AT_L1I_CACHESIZE: Self = Self(40);
    const AT_L1I_CACHEGEOMETRY: Self = Self(41);
    const AT_L1D_CACHESIZE: Self = Self(42);
    const AT_L1D_CACHEGEOMETRY: Self = Self(43);
    const AT_L2_CACHESIZE: Self = Self(44);
    const AT_L2_CACHEGEOMETRY: Self = Self(45);
    const AT_L3_CACHESIZE: Self = Self(46);
    const AT_L3_CACHEGEOMETRY: Self = Self(47);
    const AT_MINSIGSTKSZ: Self = Self(51);
}

#[cfg(target_os = "freebsd")]
impl Type {
    const AT_EXECPATH: Self = Self(15);
    const AT_CANARY: Self = Self(16);
    const AT_CANARYLEN: Self = Self(17);
    const AT_OSRELDATE: Self = Self(18);
    const AT_NCPUS: Self = Self(19);
    const AT_PAGESIZES: Self = Self(20);
    const AT_PAGESIZESLEN: Self = Self(21);
    const AT_TIMEKEEP: Self = Self(22);
    const AT_STACKPROT: Self = Self(23);
    const AT_EHDRFLAGS: Self = Self(24);
    const AT_HWCAP: Self = Self(25);
    const AT_BSDFLAGS: Self = Self(27);
    const AT_ARGC: Self = Self(28);
    const AT_ARGV: Self = Self(29);
    const AT_ENVC: Self = Self(30);
    const AT_ENVV: Self = Self(31);
    const AT_PS_STRINGS: Self = Self(32);
    const AT_FXRNG: Self = Self(33);
    const AT_KPRELOAD: Self = Self(34);
    const AT_USRSTACKBASE: Self = Self(35);
    const AT_USRSTACKLIM: Self = Self(36);
    const AT_COUNT: Self = Self(37);
}

impl Type {
    /// Converts the `Type` to a string.
    pub const fn to_str(self) -> &'static str {
        self.to_str_base()
            .unwrap_or_else(self.to_str_os())
            .unwrap_or("???")
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
            Self::AT_CLKTCK => "AT_CLKTCK",
            Self::AT_PLATFORM => "AT_PLATFORM",
            Self::AT_HWCAP => "AT_HWCAP",
            Self::AT_FPUCW => "AT_FPUCW",
            Self::AT_DCACHEBSIZE => "AT_DCACHEBSIZE",
            Self::AT_ICACHEBSIZE => "AT_ICACHEBSIZE",
            Self::AT_UCACHEBSIZE => "AT_UCACHEBSIZE",
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

    #[cfg(target_os = "freebsd")]
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

    #[cfg(not(any(target_os = "freebsd", target_os = "linux")))]
    const fn to_str_os(self) -> Option<&'static str> {
        None
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
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
        ptr,
        sync::atomic::{AtomicPtr, Ordering},
    };

    use cfg_if::cfg_if;

    use super::AuxVal;

    cfg_if! {
        if #[cfg(target_env = "gnu")] {
            use gnu::envp;
        } else {
            use other::envp;
        }
    }

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

    #[cfg(target_env = "gnu")]
    mod gnu {
        use core::{
            ffi::c_int,
            ptr,
            sync::atomic::{AtomicPtr, Ordering},
        };

        pub fn envp() -> *const *const u8 {
            ENVP.load(Ordering::Relaxed)
        }

        static ENVP: AtomicPtr<*const u8> = AtomicPtr::new(ptr::null_mut());

        #[link_section = ".init_array.00099"]
        #[used]
        static ARGV_INIT_ARRAY: extern "C" fn(c_int, *const *const u8, *const *const u8) = init;

        extern "C" fn init(_argc: c_int, _argv: *const *const u8, envp: *const *const u8) {
            ENVP.store(envp.cast_mut(), Ordering::Relaxed);
        }
    }

    #[cfg(not(target_env = "gnu"))]
    mod other {
        extern "C" {
            static mut environ: *const *const c_char;
        }

        pub fn envp() -> *const *const u8 {
            environ
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(target_os = "linux")]
    fn it_works() {
        let v = super::AuxVec::from_static();
        println!("{v}");
    }
}
