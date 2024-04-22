use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    if os == "freebsd" || os == "dragonfly" {
        use_feature("freebsdish");
    } else if os == "solaris" || os == "illumos" {
        use_feature("solarish");
    } else if os == "linux" {
        use_feature("linuxish");
    } else if os == "netbsd" {
        use_feature("netbsdish");
    }
}

fn use_feature(feature: &str) {
    println!("cargo:rustc-cfg={}", feature);
}
