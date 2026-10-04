/// Contains a single boolean which determines whether or not debug logging will
/// be done.
#[derive(Debug, Default)]
pub struct ChrnLogger {
    can_log: bool,
}

impl ChrnLogger {
    pub const fn new(can_log: bool) -> ChrnLogger {
        ChrnLogger { can_log }
    }

    /// Prints msg with [DBG] header
    pub fn log_dbg<F, T>(&self, f: F)
    where
        F: Fn() -> T,
        T: std::fmt::Display,
    {
        if self.can_log {
            println!("[DBG] {}", f())
        }
    }

    /// Prints msg with [WRN] header
    pub fn log_warn<F, T>(&self, f: F)
    where
        F: Fn() -> T,
        T: std::fmt::Display,
    {
        if self.can_log {
            println!("[WRN] {}", f())
        }
    }

    /// Prints msg with [ERR] header
    pub fn log_err<F, T>(&self, f: F)
    where
        F: Fn() -> T,
        T: std::fmt::Display,
    {
        if self.can_log {
            println!("[ERR] {}", f())
        }
    }
}
