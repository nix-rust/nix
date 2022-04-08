fn main() {
    let cfg = autocfg::new();

    if cfg.probe_rustc_version(1, 52) {
        autocfg::emit("has_doc_alias");
    }
}
