extern crate gcc;

use std::env;

pub fn main() {
    let target = env::var("TARGET").unwrap();

    let os = if target.contains("linux") {
        "LINUX"
    } else if target.contains("darwin") {
        "DARWIN"
    } else {
        "UNKNOWN"
    };

    let mut config = gcc::Config::new();

    for file in &["src/const.c", "src/sizes.c"] {
        config.file(file);
    }

    config.define(os, None);

    config.compile("libnixtest.a");
}

/*
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

    let res = Command::new("make")
        .arg("-f").arg(&make)
        .current_dir(&out)
        .env("VPATH", &src)
        .env("OS", os)
        .spawn().unwrap()
        .wait().unwrap();

    assert!(res.success());

    println!("cargo:rustc-flags=-L {}/", out);
}
*/
