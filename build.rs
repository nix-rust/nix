#[cfg(target_os = "dragonfly")]
extern crate cc;
extern crate version_check as rustc;

fn main() {
    #[cfg(target_os = "dragonfly")]
    cc::Build::new()
        .file("src/errno_dragonfly.c")
        .compile("liberrno_dragonfly.a");

    if rustc::is_min_version("1.34.0").unwrap_or(false) {
        println!("cargo:rustc-cfg=try_from");
    }
}
