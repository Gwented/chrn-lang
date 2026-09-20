use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::id_types::{InternedId, PathId};

// Um
pub const INTERNED_SELF: u32 = 0;
pub const INTERNED_STRUCT: u32 = 1;
pub const INTERNED_ENUM: u32 = 2;
pub const INTERNED_IMPORT: u32 = 3;
pub const INTERNED_EXPORT: u32 = 4;
pub const INTERNED_BIND: u32 = 5;
pub const INTERNED_ALIAS: u32 = 6;
pub const INTERNED_LET: u32 = 7;
pub const INTERNED_CHANGE: u32 = 8;
pub const INTERNED_AS: u32 = 9;
pub const INTERNED_VAR: u32 = 10;
pub const INTERNED_NEST: u32 = 11;
pub const INTERNED_COMPLEX: u32 = 12;
pub const INTERNED_OVERRIDE: u32 = 13;
pub const INTERNED_TRUE: u32 = 14;
pub const INTERNED_FALSE: u32 = 15;
pub const INTERNED_IS_EMPTY: u32 = 16;
pub const INTERNED_IS_WHITESPACE: u32 = 17;
pub const INTERNED_RANGE: u32 = 18;
pub const INTERNED_STARTSW: u32 = 19;
pub const INTERNED_ENDSW: u32 = 20;
pub const INTERNED_CONTAINS: u32 = 21;
pub const INTERNED_EQUALS: u32 = 22;
pub const INTERNED_I8: u32 = 23;
pub const INTERNED_U8: u32 = 24;
pub const INTERNED_I16: u32 = 25;
pub const INTERNED_U16: u32 = 26;
pub const INTERNED_F16: u32 = 27;
pub const INTERNED_I32: u32 = 28;
pub const INTERNED_U32: u32 = 29;
pub const INTERNED_F32: u32 = 30;
pub const INTERNED_I64: u32 = 31;
pub const INTERNED_U64: u32 = 32;
pub const INTERNED_F64: u32 = 33;
pub const INTERNED_I128: u32 = 34;
pub const INTERNED_U128: u32 = 35;
pub const INTERNED_F128: u32 = 36;
pub const INTERNED_SIZED: u32 = 37;
pub const INTERNED_UNSIZED: u32 = 38;
pub const INTERNED_BOOL: u32 = 39;
pub const INTERNED_NIL: u32 = 40;
pub const INTERNED_CHAR: u32 = 41;
pub const INTERNED_STR: u32 = 42;
pub const INTERNED_BIGINT: u32 = 43;
pub const INTERNED_BIGFLOAT: u32 = 44;
pub const INTERNED_LIST: u32 = 45;
pub const INTERNED_SET: u32 = 46;
pub const INTERNED_MAP: u32 = 47;
pub const INTERNED_TUPLE: u32 = 48;
pub const INTERNED_RUNTIME: u32 = 49;
pub const INTERNED_CORE: u32 = 50;
pub const INTERNED_IN: u32 = 51;
pub const INTERNED_RANGED: u32 = 52;
pub const INTERNED_CHARACTER_MAPPABLE: u32 = 53;
pub const INTERNED_COLLECTION: u32 = 54;
pub const INTERNED_HAS_LEN: u32 = 55;
pub const INTERNED_INTEGER: u32 = 56;
pub const INTERNED_NUMERIC: u32 = 57;
pub const INTERNED_SIGNED_INTEGER: u32 = 58;
pub const INTERNED_UNSIGNED_INTEGER: u32 = 59;
pub const INTERNED_FLOAT: u32 = 60;
pub const INTERNED_ORDERED: u32 = 61;
pub const INTERNED_COMPARABLE: u32 = 62;
pub const INTERNED_JAVA_UPPER: u32 = 63;
pub const INTERNED_DEFAULT_VAL: u32 = 64;
pub const INTERNED_WARN: u32 = 65;
pub const INTERNED_IGNORE: u32 = 66;
pub const INTERNED_SCIENT: u32 = 67;
pub const INTERNED_HEX: u32 = 68;
pub const INTERNED_BIN: u32 = 69;
pub const INTERNED_OCTAL: u32 = 70;
pub const INTERNED_IDENTS: u32 = 71;
pub const INTERNED_CASES: u32 = 72;
pub const INTERNED_JAVA_LOWER: u32 = 73;
pub const INTERNED_INT: u32 = 74;
pub const INTERNED_UNICODE: u32 = 75;
pub const INTERNED_UNKNOWN: u32 = 76;
pub const INTERNED_TYPES_LOWER: u32 = 77;
pub const INTERNED_MAX_UPPER: u32 = 78;
pub const INTERNED_MIN_UPPER: u32 = 79;
pub const INTERNED_FOR: u32 = 80;
pub const INTERNED_LONG: u32 = 81;
pub const INTERNED_SHORT: u32 = 82;
pub const INTERNED_BYTE: u32 = 83;
pub const INTERNED_FLOAT_LOWER: u32 = 84;
pub const INTERNED_DOUBLE: u32 = 85;
pub const INTERNED_BOOLEAN: u32 = 86;
pub const INTERNED_STRING: u32 = 87;
pub const INTERNED_USIZE: u32 = 88;
pub const INTERNED_ISIZE: u32 = 89;
pub const INTERNED_RUST_UPPER: u32 = 90;
pub const INTERNED_RUST_LOWER: u32 = 91;
pub const INTERNED_PI_UPPER: u32 = 92;
pub const INTERNED_E_UPPER: u32 = 93;
pub const INTERNED_TAU: u32 = 94;
pub const INTERNED_FRAC_1_PI: u32 = 95;
pub const INTERNED_FRAC_1_SQRT_2: u32 = 96;
pub const INTERNED_FRAC_2_PI: u32 = 97;
pub const INTERNED_FRAC_2_SQRT_PI: u32 = 98;
pub const INTERNED_FRAC_PI_2: u32 = 99;
pub const INTERNED_FRAC_PI_3: u32 = 100;
pub const INTERNED_FRAC_PI_4: u32 = 101;
pub const INTERNED_FRAC_PI_6: u32 = 102;
pub const INTERNED_FRAC_PI_8: u32 = 103;
pub const INTERNED_LN_2: u32 = 104;
pub const INTERNED_LN_10: u32 = 105;
pub const INTERNED_LOG2_10: u32 = 106;
pub const INTERNED_LOG2_E: u32 = 107;
pub const INTERNED_LOG10_2: u32 = 108;
pub const INTERNED_LOG10_E: u32 = 109;
pub const INTERNED_SQRT_2: u32 = 110;
pub const INTERNED_GOLDEN_RATIO: u32 = 111;
pub const INTERNED_EULER_GAMMA: u32 = 112;
pub const INTERNED_BITS_UPPER: u32 = 113;
pub const INTERNED_BYTES_UPPER: u32 = 114;
pub const INTERNED_RADIX: u32 = 115;
pub const INTERNED_DIGITS: u32 = 116;
pub const INTERNED_MANTISSA_DIGITS: u32 = 117;
pub const INTERNED_EPSILON: u32 = 118;
pub const INTERNED_INF: u32 = 119;
pub const INTERNED_NEG_INF: u32 = 120;
pub const INTERNED_NAN: u32 = 121;
pub const INTERNED_MIN_POSITIVE: u32 = 122;
pub const INTERNED_SQRT_3: u32 = 123;
pub const INTERNED_SIGN_BITS: u32 = 124;
pub const INTERNED_EXPONENT_BITS: u32 = 125;
pub const INTERNED_SIGNIFICAND_BITS: u32 = 126;
pub const INTERNED_STORED_SIGNIFICAND_BITS: u32 = 127;
pub const INTERNED_EXPONENT_BIAS: u32 = 128;
pub const INTERNED_MIN_NORMAL_EXPONENT: u32 = 129;
pub const INTERNED_MAX_NORMAL_EXPONENT: u32 = 130;
pub const INTERNED_MIN_SUBNORMAL_EXPONENT: u32 = 131;
pub const INTERNED_NUL: u32 = 132;
pub const INTERNED_SPACE_UPPER: u32 = 133;
pub const INTERNED_TAB_UPPER: u32 = 134;
pub const INTERNED_NEWLINE_UPPER: u32 = 135;
pub const INTERNED_CARRIAGE_RETURN_UPPER: u32 = 136;
pub const INTERNED_REPLACEMENT_CHARACTER_UPPER: u32 = 137;
pub const INTERNED_TAB_VALUE: u32 = 138;
pub const INTERNED_NEWLINE_VALUE: u32 = 139;
pub const INTERNED_CARRIAGE_RETURN_VALUE: u32 = 140;
pub const INTERNED_UTF8_MAX_BYTES: u32 = 141;
pub const INTERNED_UTF8_MIN_BYTES: u32 = 142;
pub const INTERNED_UTF8_MAX_BITS: u32 = 143;
pub const INTERNED_UTF8_MIN_BITS: u32 = 144;
pub const INTERNED_UTF16_MAX_BYTES: u32 = 145;
pub const INTERNED_UTF16_MIN_BYTES: u32 = 146;
pub const INTERNED_UTF16_MAX_BITS: u32 = 147;
pub const INTERNED_UTF16_MIN_BITS: u32 = 148;
pub const INTERNED_UTF16_MAX_CODE_UNITS: u32 = 149;
pub const INTERNED_UTF16_MIN_CODE_UNITS: u32 = 150;
pub const INTERNED_UTF32_BYTES: u32 = 151;
pub const INTERNED_UTF32_BITS: u32 = 152;

// Collection,
// CharacterMappable,
// HasLen,
// Ranged,
// Ordered,
// Comparable,
// Numeric,
// Integer,
// Float,
// Bool,
// Str,
// Char,
// Nil,
// MAKE THE MACRO PLEASE
// What macro. What is a macro? What is hygiene?

/// Every compiler known interned string, paired with its id. Order must match the ids.
pub static PRELOADED_STRINGS: [(&str, u32); INTERNER_PRELOAD_SIZE] = [
    ("self", INTERNED_SELF),
    ("struct", INTERNED_STRUCT),
    ("enum", INTERNED_ENUM),
    ("import", INTERNED_IMPORT),
    ("export", INTERNED_EXPORT),
    ("bind", INTERNED_BIND),
    ("alias", INTERNED_ALIAS),
    ("let", INTERNED_LET),
    ("change", INTERNED_CHANGE),
    ("as", INTERNED_AS),
    ("var", INTERNED_VAR),
    ("nest", INTERNED_NEST),
    ("complex", INTERNED_COMPLEX),
    ("override", INTERNED_OVERRIDE),
    ("true", INTERNED_TRUE),
    ("false", INTERNED_FALSE),
    ("IsEmpty", INTERNED_IS_EMPTY),
    ("IsWhitespace", INTERNED_IS_WHITESPACE),
    ("Range", INTERNED_RANGE),
    ("StartsW", INTERNED_STARTSW),
    ("EndsW", INTERNED_ENDSW),
    ("Contains", INTERNED_CONTAINS),
    ("Equals", INTERNED_EQUALS),
    ("i8", INTERNED_I8),
    ("u8", INTERNED_U8),
    ("i16", INTERNED_I16),
    ("u16", INTERNED_U16),
    ("f16", INTERNED_F16),
    ("i32", INTERNED_I32),
    ("u32", INTERNED_U32),
    ("f32", INTERNED_F32),
    ("i64", INTERNED_I64),
    ("u64", INTERNED_U64),
    ("f64", INTERNED_F64),
    ("i128", INTERNED_I128),
    ("u128", INTERNED_U128),
    ("f128", INTERNED_F128),
    ("sized", INTERNED_SIZED),
    ("unsized", INTERNED_UNSIZED),
    ("bool", INTERNED_BOOL),
    ("nil", INTERNED_NIL),
    ("char", INTERNED_CHAR),
    ("str", INTERNED_STR),
    ("BigInt", INTERNED_BIGINT),
    ("BigFloat", INTERNED_BIGFLOAT),
    ("List", INTERNED_LIST),
    ("Set", INTERNED_SET),
    ("Map", INTERNED_MAP),
    ("Tuple", INTERNED_TUPLE),
    ("Runtime", INTERNED_RUNTIME),
    ("core", INTERNED_CORE),
    ("in", INTERNED_IN),
    ("Ranged", INTERNED_RANGED),
    ("CharacterMappable", INTERNED_CHARACTER_MAPPABLE),
    ("Collection", INTERNED_COLLECTION),
    ("HasLen", INTERNED_HAS_LEN),
    ("Integer", INTERNED_INTEGER),
    ("Numeric", INTERNED_NUMERIC),
    ("SignedInteger", INTERNED_SIGNED_INTEGER),
    ("UnsignedInteger", INTERNED_UNSIGNED_INTEGER),
    ("Float", INTERNED_FLOAT),
    ("Ordered", INTERNED_ORDERED),
    ("Comparable", INTERNED_COMPARABLE),
    ("JAVA", INTERNED_JAVA_UPPER),
    ("default_val", INTERNED_DEFAULT_VAL),
    ("warn", INTERNED_WARN),
    ("ignore", INTERNED_IGNORE),
    ("scient", INTERNED_SCIENT),
    ("hex", INTERNED_HEX),
    ("bin", INTERNED_BIN),
    ("octal", INTERNED_OCTAL),
    ("idents", INTERNED_IDENTS),
    ("cases", INTERNED_CASES),
    ("java", INTERNED_JAVA_LOWER),
    ("int", INTERNED_INT),
    ("unicode", INTERNED_UNICODE),
    ("Unknown", INTERNED_UNKNOWN),
    ("types", INTERNED_TYPES_LOWER),
    ("MAX", INTERNED_MAX_UPPER),
    ("MIN", INTERNED_MIN_UPPER),
    ("for", INTERNED_FOR),
    ("long", INTERNED_LONG),
    ("short", INTERNED_SHORT),
    ("byte", INTERNED_BYTE),
    ("float", INTERNED_FLOAT_LOWER),
    ("double", INTERNED_DOUBLE),
    ("boolean", INTERNED_BOOLEAN),
    ("String", INTERNED_STRING),
    ("usize", INTERNED_USIZE),
    ("isize", INTERNED_ISIZE),
    ("RUST", INTERNED_RUST_UPPER),
    ("rust", INTERNED_RUST_LOWER),
    ("PI", INTERNED_PI_UPPER),
    ("E", INTERNED_E_UPPER),
    ("TAU", INTERNED_TAU),
    ("FRAC_1_PI", INTERNED_FRAC_1_PI),
    ("FRAC_1_SQRT_2", INTERNED_FRAC_1_SQRT_2),
    ("FRAC_2_PI", INTERNED_FRAC_2_PI),
    ("FRAC_2_SQRT_PI", INTERNED_FRAC_2_SQRT_PI),
    ("FRAC_PI_2", INTERNED_FRAC_PI_2),
    ("FRAC_PI_3", INTERNED_FRAC_PI_3),
    ("FRAC_PI_4", INTERNED_FRAC_PI_4),
    ("FRAC_PI_6", INTERNED_FRAC_PI_6),
    ("FRAC_PI_8", INTERNED_FRAC_PI_8),
    ("LN_2", INTERNED_LN_2),
    ("LN_10", INTERNED_LN_10),
    ("LOG2_10", INTERNED_LOG2_10),
    ("LOG2_E", INTERNED_LOG2_E),
    ("LOG10_2", INTERNED_LOG10_2),
    ("LOG10_E", INTERNED_LOG10_E),
    ("SQRT_2", INTERNED_SQRT_2),
    ("GOLDEN_RATIO", INTERNED_GOLDEN_RATIO),
    ("EULER_GAMMA", INTERNED_EULER_GAMMA),
    ("BITS", INTERNED_BITS_UPPER),
    ("BYTES", INTERNED_BYTES_UPPER),
    ("RADIX", INTERNED_RADIX),
    ("DIGITS", INTERNED_DIGITS),
    ("MANTISSA_DIGITS", INTERNED_MANTISSA_DIGITS),
    ("EPSILON", INTERNED_EPSILON),
    ("INF", INTERNED_INF),
    ("NEG_INF", INTERNED_NEG_INF),
    ("NAN", INTERNED_NAN),
    ("MIN_POSITIVE", INTERNED_MIN_POSITIVE),
    ("SQRT_3", INTERNED_SQRT_3),
    ("SIGN_BITS", INTERNED_SIGN_BITS),
    ("EXPONENT_BITS", INTERNED_EXPONENT_BITS),
    ("SIGNIFICAND_BITS", INTERNED_SIGNIFICAND_BITS),
    ("STORED_SIGNIFICAND_BITS", INTERNED_STORED_SIGNIFICAND_BITS),
    ("EXPONENT_BIAS", INTERNED_EXPONENT_BIAS),
    ("MIN_NORMAL_EXPONENT", INTERNED_MIN_NORMAL_EXPONENT),
    ("MAX_NORMAL_EXPONENT", INTERNED_MAX_NORMAL_EXPONENT),
    ("MIN_SUBNORMAL_EXPONENT", INTERNED_MIN_SUBNORMAL_EXPONENT),
    ("NUL", INTERNED_NUL),
    ("SPACE", INTERNED_SPACE_UPPER),
    ("TAB", INTERNED_TAB_UPPER),
    ("NEWLINE", INTERNED_NEWLINE_UPPER),
    ("CARRIAGE_RETURN", INTERNED_CARRIAGE_RETURN_UPPER),
    (
        "REPLACEMENT_CHARACTER",
        INTERNED_REPLACEMENT_CHARACTER_UPPER,
    ),
    ("\t", INTERNED_TAB_VALUE),
    ("\n", INTERNED_NEWLINE_VALUE),
    ("\r", INTERNED_CARRIAGE_RETURN_VALUE),
    ("UTF8_MAX_BYTES", INTERNED_UTF8_MAX_BYTES),
    ("UTF8_MIN_BYTES", INTERNED_UTF8_MIN_BYTES),
    ("UTF8_MAX_BITS", INTERNED_UTF8_MAX_BITS),
    ("UTF8_MIN_BITS", INTERNED_UTF8_MIN_BITS),
    ("UTF16_MAX_BYTES", INTERNED_UTF16_MAX_BYTES),
    ("UTF16_MIN_BYTES", INTERNED_UTF16_MIN_BYTES),
    ("UTF16_MAX_BITS", INTERNED_UTF16_MAX_BITS),
    ("UTF16_MIN_BITS", INTERNED_UTF16_MIN_BITS),
    ("UTF16_MAX_CODE_UNITS", INTERNED_UTF16_MAX_CODE_UNITS),
    ("UTF16_MIN_CODE_UNITS", INTERNED_UTF16_MIN_CODE_UNITS),
    ("UTF32_BYTES", INTERNED_UTF32_BYTES),
    ("UTF32_BITS", INTERNED_UTF32_BITS),
];

/// Interner used for the chrn language
#[derive(Debug)]
pub struct Intern {
    // Um
    id_map: HashMap<String, u32>,
    path_map: HashMap<PathBuf, u32>,
    // Is super solely for lib.rs tests
    pub(super) stored_strs: Vec<String>,
    stored_paths: Vec<PathBuf>,
    // Maybe not
    pos: usize,
}

pub const INTERNER_PRELOAD_SIZE: usize = (INTERNED_UTF32_BITS + 1) as usize;

impl Intern {
    /// Creates interner that pre-loads itself with all defined interned string literals.
    pub fn init() -> Intern {
        let mut interner = Intern {
            id_map: HashMap::with_capacity(INTERNER_PRELOAD_SIZE),
            stored_strs: Vec::with_capacity(INTERNER_PRELOAD_SIZE),
            path_map: HashMap::new(),
            stored_paths: Vec::new(),
            pos: 0,
        };

        // Pre-loading every language required string literal
        for (s, id) in PRELOADED_STRINGS {
            interner.register(s, id);
        }

        interner.pos = interner.stored_strs.len();

        interner
    }

    /// Internal helper used for registering compiler known interned strings
    fn register(&mut self, s: &str, id: u32) {
        debug_assert_eq!(self.stored_strs.len() as u32, id);
        self.id_map.insert(s.to_string(), id);
        self.stored_strs.push(s.to_string());
    }

    pub fn intern(&mut self, s: &str) -> InternedId {
        if let Some(id) = self.id_map.get(s) {
            return InternedId::new(*id);
        }

        let id = self.stored_strs.len() as u32;
        self.pos += 1;

        let new_str = s.to_string();

        self.id_map.insert(new_str.clone(), id);
        self.stored_strs.push(new_str);

        InternedId::new(id)
    }

    /// Method for `self` to intern all of `other`'s stored strings and paths
    pub fn append(&mut self, other: &Intern) {
        for i in INTERNER_PRELOAD_SIZE..other.stored_strs.len() {
            let current = &other.stored_strs[i];
            self.intern(current);
        }

        for i in 0..other.stored_paths.len() {
            let current = &other.stored_paths[i];
            self.intern_path(current);
        }
    }

    pub fn intern_path(&mut self, s: &Path) -> PathId {
        if let Some(id) = self.path_map.get(s) {
            return PathId::new(*id);
        }

        let id = self.stored_paths.len() as u32;
        self.pos += 1;

        let new_path = s.to_path_buf();

        self.path_map.insert(new_path.clone(), id);
        self.stored_paths.push(new_path);

        PathId::new(id)
    }

    pub fn search(&self, interned_id: InternedId) -> &str {
        &self.stored_strs[interned_id.id as usize]
    }

    pub fn search_idx(&self, idx: usize) -> &str {
        &self.stored_strs[idx]
    }

    pub fn try_search_str(&self, s: &str) -> Option<InternedId> {
        self.id_map.get(s).map(|id| InternedId::new(*id))
    }

    pub fn search_path(&self, path_id: PathId) -> &Path {
        &self.stored_paths[path_id.id as usize]
    }

    pub fn search_direct_path(&self, path: &Path) -> Option<&Path> {
        if let Some(id) = self.path_map.get(path) {
            return Some(self.search_path(PathId::new(*id)));
        }

        None
    }
}
