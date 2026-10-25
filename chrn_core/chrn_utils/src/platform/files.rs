// This is cool
pub mod error;

#[cfg(unix)]
pub mod unix;
#[cfg(unix)]
pub use unix::*;

//TODO: SAVE ME
#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(not(any(unix, windows)))]
pub mod unknown;
#[cfg(not(any(unix, windows)))]
pub use unknown::*;
