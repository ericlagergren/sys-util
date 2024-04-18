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
        target_os = "linux",
        target_os = "netbsd",
        target_os = "solaris",
    ))]
    #[cfg_attr(
        docs,
        doc(cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
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
    const AT_NULL: Type = Type(0);
    const AT_IGNORE: Type = Type(1);
    const AT_EXECFD: Type = Type(2);
    const AT_PHDR: Type = Type(3);
    const AT_PHENT: Type = Type(4);
    const AT_PHNUM: Type = Type(5);
    const AT_PAGESZ: Type = Type(6);
    const AT_BASE: Type = Type(7);
    const AT_FLAGS: Type = Type(8);
    const AT_ENTRY: Type = Type(9);
    const AT_NOTELF: Type = Type(10);
    const AT_UID: Type = Type(11);
    const AT_EUID: Type = Type(12);
    const AT_GID: Type = Type(13);
    const AT_EGID: Type = Type(14);
    const AT_CLKTCK: Type = Type(17);

    const AT_PLATFORM: Type = Type(15);
    const AT_HWCAP: Type = Type(16);

    const AT_FPUCW: Type = Type(18);

    const AT_DCACHEBSIZE: Type = Type(19);
    const AT_ICACHEBSIZE: Type = Type(20);
    const AT_UCACHEBSIZE: Type = Type(21);

    const AT_IGNOREPPC: Type = Type(22);

    const AT_SECURE: Type = Type(23);

    const AT_BASE_PLATFORM: Type = Type(24);

    const AT_RANDOM: Type = Type(25);

    const AT_HWCAP2: Type = Type(26);

    const AT_RSEQ_FEATURE_SIZE: Type = Type(27);
    const AT_RSEQ_ALIGN: Type = Type(28);

    const AT_HWCAP3: Type = Type(29);
    const AT_HWCAP4: Type = Type(30);

    const AT_EXECFN: Type = Type(31);

    const AT_SYSINFO: Type = Type(32);
    const AT_SYSINFO_EHDR: Type = Type(33);

    const AT_L1I_CACHESHAPE: Type = Type(34);
    const AT_L1D_CACHESHAPE: Type = Type(35);
    const AT_L2_CACHESHAPE: Type = Type(36);
    const AT_L3_CACHESHAPE: Type = Type(37);

    const AT_L1I_CACHESIZE: Type = Type(40);
    const AT_L1I_CACHEGEOMETRY: Type = Type(41);
    const AT_L1D_CACHESIZE: Type = Type(42);
    const AT_L1D_CACHEGEOMETRY: Type = Type(43);
    const AT_L2_CACHESIZE: Type = Type(44);
    const AT_L2_CACHEGEOMETRY: Type = Type(45);
    const AT_L3_CACHESIZE: Type = Type(46);
    const AT_L3_CACHEGEOMETRY: Type = Type(47);

    const AT_MINSIGSTKSZ: Type = Type(51);

    /// Converts the `Type` to a string.
    pub const fn to_str(self) -> &'static str {
        match self {
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
            Self::AT_HWCAP2 => "AT_HWCAP2",
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
            _ => "???",
        }
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
