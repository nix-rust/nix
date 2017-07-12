extern crate nix;

use std::io::{Error as IOError, ErrorKind as IOErrorKind};
use std::io::Write;
use nix::{Error, Result as NixResult};
use nix::errno::Errno;

fn generate_io_error_os(os_error_value: i32) -> Result<(), IOError> {
    Err(IOError::from_raw_os_error(os_error_value))
}

fn generate_io_error_kind(error_kind: IOErrorKind) -> Result<(), IOError> {
    Err(IOError::new(error_kind, "test error"))
}

fn try_error_function<FError: FnOnce() -> Result<(), IOError>>(error_fn: FError) -> NixResult<()> {
    // the main goal of From<io::Error> for nix::Error is to be able to implicitly convert
    // an std::io::Error into a nix::Error, enabling ? and try! for std::io functions.
    error_fn()?;
    Ok(())
}

#[test]
fn test_io_error_to_nix_error() {
    // testing that an errno error can be converted
    let error_os_value = 1;
    let error_os = try_error_function(|| generate_io_error_os(error_os_value));

    assert!(error_os.is_err());
    assert_eq!(error_os.unwrap_err(), Error::Sys(Errno::from_i32(error_os_value)));

    // testing that an IOErrorKind can be converted
    let error_kind_value = IOErrorKind::ConnectionReset;
    let error_kind = try_error_function(|| generate_io_error_kind(error_kind_value));

    assert!(error_kind.is_err());
    assert_eq!(error_kind.unwrap_err(), Error::IOError(error_kind_value));
}

#[test]
fn test_io_error_display() {
    // testing that the IOError can be displayed
    let error_value = IOErrorKind::ConnectionReset;
    let error = try_error_function(|| generate_io_error_kind(error_value)).unwrap_err();
    let mut error_display = Vec::new();

    assert!(write!(&mut error_display, "{}", error).is_ok());
    assert_eq!(String::from_utf8(error_display).unwrap(), "IO Error: ConnectionReset");
}
