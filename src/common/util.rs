use std::ffi::CString;

use super::Error;

/// Helper for creating CStrings.
pub fn make_cstring(value: impl Into<Vec<u8>>) -> Result<CString, Error> {
    CString::new(value).map_err(|_| Error::ValueError("CString::new failed".to_owned()))
}