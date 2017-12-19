#[cfg(target_os = "dragonfly")]
extern crate gcc;

#[cfg(target_os = "dragonfly")]
fn main() {
    gcc::Build::new()
        .file("src/errno_dragonfly.c")
        .compile("liberrno_dragonfly.a");
}

#[cfg(not(target_os = "dragonfly"))]
fn main() {}
