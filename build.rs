use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        ios: { target_os = "ios" },
        macos: { target_os = "macos" },
        watchos: { target_os = "watchos" },
        tvos: { target_os = "tvos" },
        apple_targets: { any(ios, macos, watchos, tvos) },
    }
}
