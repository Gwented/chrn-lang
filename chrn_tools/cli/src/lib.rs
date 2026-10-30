pub mod args;
pub mod config;
mod detect;
pub mod dispatcher;
mod env_vars;
mod files;
mod macros;
mod presentation;
mod renderer;

// Argument to nullify this should exist
/// Max diagnostics that can be held by the reporter
const MAX_DIAGNOSTICS: usize = 80;

// This is a bit odd since u32 is the max usable in the compiler, meaning if this were wrong (it
// wont be this is a fake issue) this is a fake issue. I am the issue maker.
/// Max value `max-numeric-bits` can be given
const MAX_NUMERIC_BITS: u32 = u32::MAX;

#[cfg(test)]
mod tests;
