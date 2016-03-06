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

    gcc::Config::new()
        .file("src/const.c")
        .file("src/sizes.c")
        .define(os, None)
        .compile("libnixtest.a");
}
