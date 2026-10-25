use std::{path::PathBuf, str::FromStr};

use crate::platform::files::error::PathConversionErrorKind;

/// Attempts to UTF-8 validate bytes. Returns `PathConversionErrorKind` on failure.
pub fn pathbuf_from_bytes(bytes: &[u8]) -> Result<PathBuf, PathConversionErrorKind> {
    match str::from_utf8(bytes) {
        Ok(s) => Ok(PathBuf::from_str(&s).expect("Infallible")),
        Err(_) => Err(PathConversionErrorKind::InvalidUTF8),
    }
}
