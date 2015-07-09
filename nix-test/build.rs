use std::env;
use std::process::Command;

pub fn main() {
    let root   = env::var("CARGO_MANIFEST_DIR").unwrap();
    let make   = root.clone() + "/Makefile";
    let src    = root.clone() + "/src";
    let out    = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();

    let os = if target.contains("linux") {
        "LINUX"
    } else if target.contains("darwin") {
        "DARWIN"
    } else {
        "UNKNOWN"
    };

    let arch = if target.contains("i686") || target.contains("i386") || target.contains("arm") || target.contains("mips") || target.contains("powerpc") {
        "32"
    } else if target.contains("x86_64") || target.contains("aarch64") {
        "64"
    } else {
        "64" // TODO decide a better strategy
    };

    let res = Command::new("make")
        .arg("-f").arg(&make)
        .current_dir(&out)
        .env("VPATH", &src)
        .env("OS", os)
        .env("ARCH", arch)
        .spawn().unwrap()
        .wait().unwrap();

    assert!(res.success());

    println!("cargo:rustc-flags=-L {}/", out);
}
