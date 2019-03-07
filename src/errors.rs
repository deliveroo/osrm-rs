use std::error;
use std::ffi::{self, CStr};
use std::fmt::{self, Debug, Display};
use std::result::Result as StdResult;

struct OsrmcError {
    handle: osrmc_sys::osrmc_error_t,
}

impl_drop!(OsrmcError, osrmc_sys::osrmc_error_destruct);

// This is just a thin wrapper around a std::string.
unsafe impl Send for OsrmcError {}
unsafe impl Sync for OsrmcError {}

impl Display for OsrmcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        unsafe {
            let ptr = osrmc_sys::osrmc_error_message(self.handle);
            write!(f, "OsrmcError: {}", CStr::from_ptr(ptr).to_string_lossy())
        }
    }
}

impl Debug for OsrmcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        Display::fmt(self, f)
    }
}

#[derive(Debug)]
enum ErrorKind {
    Message(String),
    Osrmc(OsrmcError),
    FfiNul(ffi::NulError),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        match self {
            ErrorKind::Message(inner) => Display::fmt(inner, f),
            ErrorKind::Osrmc(inner) => Display::fmt(inner, f),
            ErrorKind::FfiNul(inner) => Display::fmt(inner, f),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        write!(f, "osrm-rs: {}", self.kind)
    }
}

impl error::Error for Error {}

impl From<osrmc_sys::osrmc_error_t> for Error {
    fn from(handle: osrmc_sys::osrmc_error_t) -> Error {
        Error {
            kind: ErrorKind::Osrmc(OsrmcError { handle }),
        }
    }
}

impl From<ffi::NulError> for Error {
    fn from(other: ffi::NulError) -> Error {
        Error {
            kind: ErrorKind::FfiNul(other),
        }
    }
}

impl From<String> for Error {
    fn from(other: String) -> Error {
        Error {
            kind: ErrorKind::Message(other),
        }
    }
}

impl From<&str> for Error {
    fn from(other: &str) -> Error {
        Error {
            kind: ErrorKind::Message(other.into()),
        }
    }
}

pub type Result<T> = StdResult<T, Error>;
