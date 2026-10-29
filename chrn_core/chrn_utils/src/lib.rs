pub mod arena;
pub mod budget;
pub mod core_error;
pub mod err_codes;
pub mod files;
pub mod help_model;
pub mod id_types;
pub mod intern;
pub mod macros;
pub mod pair;
pub mod platform;
pub mod source_map;
pub mod utils;

// -- Heuristic max amounts for `chrn` to abide by for safety purposes --

/// Max loops before what would be considered a broken mutation loop.
/// Arbitrarily high number to help examine recursive bugs better and for stopping infinite loops in
/// general.
pub const MAX_LOOPS: u32 = 10000004;

/// Max modules that can be in memory at once
pub const MAX_MODULES: u16 = 800;

/// Max recursive descent that can be done
pub const MAX_RECURSIVE_DEPTH: u16 = 1024;

/// Max expression nodes that can be consumed in a singule expression tree
pub const MAX_EXPR_NODES: u32 = 1; // 5,000,000

//WARN: 32Kib -> 64Kib
/// Max size of any given region. This has nothing to do with file size overall, just how many bytes
/// a given region can occupy, which may be a full file or just from `@def` to `@end`.
pub const MAX_REGION_SIZE: usize = 1024 * 64;

/// File extension for `chrn`
pub const CHRN_FILE_EXTENSION: &str = "chrn";

//NOTE: If concat is added then a high value for max string length can prevent over-allocation in
//that particular scenario. This doesn't have to be accounted for right now since there is no concat
//so a string can only be as large as a region, which is not very large.
//// Max length a string can be before the program reacts
// pub const MAX_STR_LEN: usize = usize::MAX - 8;

#[cfg(test)]
mod tests;
