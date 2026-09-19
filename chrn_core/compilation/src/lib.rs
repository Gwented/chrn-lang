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

#[cfg(test)]
mod tests;
