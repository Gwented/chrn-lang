pub mod chrn_config;
pub mod config_loader;
pub mod constraints;
pub mod cst;
pub mod id_tag_decls;
pub mod lexer;
pub mod lookup;
pub mod macros;
pub mod module;
pub mod parser;
pub mod resolvers;
pub mod script_compiler;
pub mod semantic;

// I think this is appropriate placement?
// It IS a general language level rule, but at the same time what if the implementation was
// different? Maybe move this.
/// Max depth for config reach for `chrn`, excluding `override` section expansion
pub const CFG_MAX_COMPLEX_NEST_LEVEL: u8 = 2;

// For float and int where applicable
/// Default max bits for numeric values
pub const DEFAULT_MAX_NUMERIC_BITS: u32 = 256;

// May be larger.
/// Default max bits for numeric values
pub const INT_BITS_BEFORE_ARBITRARY: u32 = 64;

// Will not change
/// Default max bits for numeric values
pub const FLOAT_BITS_BEFORE_ARBITRARY: u32 = 64;

#[cfg(test)]
mod tests;
