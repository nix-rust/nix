extern crate rustc_version;
extern crate semver;

use semver::Version;

fn main() {
    if rustc_version::version() >= Version::parse("1.6.0").unwrap() {
        println!("cargo:rustc-cfg=raw_pointer_derive_allowed");
    }
}
