/// Helper for implementing Drop for a libosrmc handle.
macro_rules! impl_drop {
    ($ty:ident, $destructor:path) => {
        impl Drop for $ty {
            fn drop(&mut self) {
                unsafe { $destructor(self.handle) }
            }
        }
    };
}

/// Helper for calling libosrmc methods which take an error as a final parameter.
///
/// Takes care of passing in an empty error, and converts the response into a result.
macro_rules! call_with_error {
    ($func:ident($( $arg:expr ),*)) => {{
        let mut error = std::ptr::null_mut();
        let result = unsafe {
            osrmc_sys::$func($($arg,)* &mut error)
        };
        if !error.is_null() {
            Err(Error::from(error))
        } else {
            Ok(result)
        }
    }};
}
