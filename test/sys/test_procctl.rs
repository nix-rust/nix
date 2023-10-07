#[cfg(target_os = "freebsd")]
#[cfg(feature = "process")]
mod test_prctl {
    use nix::sys::proctl;

    #[test]
    fn test_get_set_dumpable() {
        let original = procctl::get_dumpable().unwrap();

        prctl::set_dumpable(false).unwrap();
        let dumpable = procctl::get_dumpable().unwrap();
        assert!(!dumpable);

        prctl::set_dumpable(original).unwrap();
    }

    #[test]
    fn test_get_set_pdeathsig() {
        use nix::sys::signal::Signal;

        let original = procctl::get_pdeathsig().unwrap();

        procctl::set_pdeathsig(Signal::SIGUSR1).unwrap();
        let sig = procctl::get_pdeathsig().unwrap();
        assert_eq!(sig, Some(Signal::SIGUSR1));

        procctl::set_pdeathsig(original).unwrap();
    }
}
