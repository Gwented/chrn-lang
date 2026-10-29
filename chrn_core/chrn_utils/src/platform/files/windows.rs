use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::{path::PathBuf, str::FromStr};

use crate::platform::files::error::PathConversionErrorKind;

//TODO: Wide conversions
/// Attempts to UTF-8 validate bytes. Returns `PathConversionErrorKind` on failure.
pub fn pathbuf_from_bytes(bytes: &[u8]) -> Result<PathBuf, PathConversionErrorKind> {
    // Windows why.
    // OsString::from_wide();
    match str::from_utf8(bytes) {
        Ok(s) => Ok(PathBuf::from_str(&s).expect("Infallible")),
        Err(_) => Err(PathConversionErrorKind::InvalidUTF8),
    }
}
