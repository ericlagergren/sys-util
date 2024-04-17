use core::{
    ffi::{c_int, CStr},
    fmt, ptr,
    sync::atomic::{AtomicIsize, AtomicPtr, Ordering},
};

#[derive(Copy, Clone, Debug)]
pub struct AuxVec([u64; 51]);

impl AuxVec {
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
}

impl AuxVec {
    pub fn new() -> Self {
		unsafe {
			show_init(
				ARGC.load(Ordering::Relaxed),
				ARGV.load(Ordering::Relaxed),
				ENVP.load(Ordering::Relaxed),
				AUXV.load(Ordering::Relaxed),
			)
		}
		Self([0u64; 51])
	}
}

impl fmt::Display for AuxVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, &v) in self.0.iter().enumerate() {
			let value = AuxVal(Type(i as u64), v);
			write!(f, "{value}")?;
		}
		Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct AuxVal(Type, u64);

impl fmt::Display for AuxVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}: {}", self.0, self.1)
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Type(u64);

impl Type {
    const fn to_str(self) -> &'static str {
        macro_rules! as_str  {
			($type:ident) => {
				AuxVec::$type => stringify!($type),
			}
		}
        match self {
            AuxVec::AT_NULL => "AT_NULL",
            AuxVec::AT_IGNORE => "AT_IGNORE",
            AuxVec::AT_EXECFD => "AT_EXECFD",
            AuxVec::AT_PHDR => "AT_PHDR",
            AuxVec::AT_PHENT => "AT_PHENT",
            AuxVec::AT_PHNUM => "AT_PHNUM",
            AuxVec::AT_PAGESZ => "AT_PAGESZ",
            AuxVec::AT_BASE => "AT_BASE",
            AuxVec::AT_FLAGS => "AT_FLAGS",
            AuxVec::AT_ENTRY => "AT_ENTRY",
            AuxVec::AT_NOTELF => "AT_NOTELF",
            AuxVec::AT_UID => "AT_UID",
            AuxVec::AT_EUID => "AT_EUID",
            AuxVec::AT_GID => "AT_GID",
            AuxVec::AT_EGID => "AT_EGID",
            AuxVec::AT_CLKTCK => "AT_CLKTCK",
            AuxVec::AT_PLATFORM => "AT_PLATFORM",
            AuxVec::AT_HWCAP => "AT_HWCAP",
            AuxVec::AT_FPUCW => "AT_FPUCW",
            AuxVec::AT_DCACHEBSIZE => "AT_DCACHEBSIZE",
            AuxVec::AT_ICACHEBSIZE => "AT_ICACHEBSIZE",
            AuxVec::AT_UCACHEBSIZE => "AT_UCACHEBSIZE",
            AuxVec::AT_IGNOREPPC => "AT_IGNOREPPC",
            AuxVec::AT_SECURE => "AT_SECURE",
            AuxVec::AT_BASE_PLATFORM => "AT_BASE_PLATFORM",
            AuxVec::AT_RANDOM => "AT_RANDOM",
            AuxVec::AT_HWCAP2 => "AT_HWCAP2",
            AuxVec::AT_RSEQ_FEATURE_SIZE => "AT_RSEQ_FEATURE_SIZE",
            AuxVec::AT_RSEQ_ALIGN => "AT_RSEQ_ALIGN",
            AuxVec::AT_HWCAP3 => "AT_HWCAP3",
            AuxVec::AT_HWCAP4 => "AT_HWCAP4",
            AuxVec::AT_EXECFN => "AT_EXECFN",
            AuxVec::AT_SYSINFO => "AT_SYSINFO",
            AuxVec::AT_SYSINFO_EHDR => "AT_SYSINFO_EHDR",
            AuxVec::AT_L1I_CACHESHAPE => "AT_L1I_CACHESHAPE",
            AuxVec::AT_L1D_CACHESHAPE => "AT_L1D_CACHESHAPE",
            AuxVec::AT_L2_CACHESHAPE => "AT_L2_CACHESHAPE",
            AuxVec::AT_L3_CACHESHAPE => "AT_L3_CACHESHAPE",
            AuxVec::AT_L1I_CACHESIZE => "AT_L1I_CACHESIZE",
            AuxVec::AT_L1I_CACHEGEOMETRY => "AT_L1I_CACHEGEOMETRY",
            AuxVec::AT_L1D_CACHESIZE => "AT_L1D_CACHESIZE",
            AuxVec::AT_L1D_CACHEGEOMETRY => "AT_L1D_CACHEGEOMETRY",
            AuxVec::AT_L2_CACHESIZE => "AT_L2_CACHESIZE",
            AuxVec::AT_L2_CACHEGEOMETRY => "AT_L2_CACHEGEOMETRY",
            AuxVec::AT_L3_CACHESIZE => "AT_L3_CACHESIZE",
            AuxVec::AT_L3_CACHEGEOMETRY => "AT_L3_CACHEGEOMETRY",
            AuxVec::AT_MINSIGSTKSZ => "AT_MINSIGSTKSZ",
            _ => "???",
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

unsafe fn show_init(
	argc: isize,
	argv: *const *const u8,
	envp: *const *const u8,
	auxv: *const AuxVal,
) {
    println!("argc = {argc}");
    println!("argv = {argv:p}");
    println!("envp = {envp:p}");
    println!("auxv = {auxv:p}");
    for i in 0..argc as isize {
        let ptr = *argv.offset(i);
        if ptr.is_null() {
            break;
        }
        let arg = CStr::from_ptr(ptr.cast()).to_str().unwrap();
        println!("#{i}: {arg}");
    }

    let mut ptr = argv.offset((argc as isize) + 1);
    while !(*ptr).is_null() {
        ptr = ptr.add(1);
    }
    ptr = ptr.add(1);
    println!("auxv = {ptr:p}");
    sysauxv(ptr.cast());
}

unsafe fn sysauxv(out: &mut [u64; 51], mut auxv: *const AuxVal) -> AuxVec {
	let mut out = [
    loop {
        let value = *auxv;
        if value.0 == AuxVec::AT_NULL {
            break;
        }
        println!("{value}");
        auxv = auxv.add(1);
    }
}

static ARGC: AtomicIsize = AtomicIsize::new(0);
static ARGV: AtomicPtr<*const u8> = AtomicPtr::new(ptr::null_mut());
static ENVP: AtomicPtr<*const u8> = AtomicPtr::new(ptr::null_mut());
//static AUXV: AtomicPtr<u64> = AtomicPtr::new(ptr::null_mut());

#[link_section = ".init_array.00099"]
#[used]
static ARGV_INIT_ARRAY: extern "C" fn(
	c_int,
	*const *const u8,
	*const *const u8,
	*const AuxVal,
) = init;

extern "C" fn init(
	argc: c_int,
	argv: *const *const u8,
	envp: *const *const u8,
	//auxv: *const AuxVal,
) {
	ARGC.store(argc as isize, Ordering::Relaxed);
	ARGV.store(argv.cast_mut(), Ordering::Relaxed);
	ENVP.store(envp.cast_mut(), Ordering::Relaxed);
	//AUXV.store(auxv.cast_mut(), Ordering::Relaxed);
}

/*
macro_rules! for_each_type {
	($($tt:tt)*) => {
	};
}
*/

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn it_works() {
		let v = AuxVec::new();
        println!("{v}");
    }
}
