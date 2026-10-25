use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::PathBuf};

use crate::platform::files::error::PathConversionErrorKind;

/// Uses `unix::ffi::OsStrExt` to convert bytes to `PathBuf`
pub fn pathbuf_from_bytes(bytes: &[u8]) -> Result<PathBuf, PathConversionErrorKind> {
    let os_str = OsStr::from_bytes(bytes);
    Ok(PathBuf::from(os_str))
}
