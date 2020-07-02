use std::borrow::Cow;
use std::error;
use std::ffi::{self, CStr};
use std::fmt::{self, Display};
use std::result::Result as StdResult;

struct OsrmcError {
    handle: osrmc_sys::osrmc_error_t,
}

impl_drop!(OsrmcError, osrmc_sys::osrmc_error_destruct);

impl OsrmcError {
    fn code(&self) -> Cow<'_, str> {
        unsafe {
            let ptr = osrmc_sys::osrmc_error_code(self.handle);
            CStr::from_ptr(ptr).to_string_lossy()
        }
    }

    fn message(&self) -> Cow<'_, str> {
        unsafe {
            let ptr = osrmc_sys::osrmc_error_message(self.handle);
            CStr::from_ptr(ptr).to_string_lossy()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
    Message(String),
    Osrmc {
        code: String,
        message: String,
    },
    NoRoute,
    InvalidCoordinate,
    FfiNul(ffi::NulError),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        match self {
            ErrorKind::Message(inner) => Display::fmt(inner, f),
            ErrorKind::Osrmc { code, message }=> write!(f, "Osrmc: {}: {}", code, message),
            ErrorKind::NoRoute => write!(f, "Impossible route between points"),
            ErrorKind::InvalidCoordinate => write!(f, "Invalid coordinate value"),
            ErrorKind::FfiNul(inner) => Display::fmt(inner, f),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        write!(f, "osrm-rs: {}", self.kind)
    }
}

impl error::Error for Error {}

impl From<osrmc_sys::osrmc_error_t> for Error {
    fn from(handle: osrmc_sys::osrmc_error_t) -> Error {
        let error = OsrmcError { handle };
        let code = error.code().into_owned();
        let message = error.message().into_owned();
        let kind = match code.as_ref() {
            "NoRoute" => ErrorKind::NoRoute,
            "InvalidValue" => ErrorKind::InvalidCoordinate,
            _ => ErrorKind::Osrmc { code, message }
        };
        Error { kind }
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
