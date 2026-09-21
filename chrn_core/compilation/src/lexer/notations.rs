//TODO: Should probably be elsewhere

// This exists so that the interned value can be kept and displayed. It's also so a notation can be
// read within the lexer and stored without losing accuracy by setting it to something like i64
/// Notation marker for Integers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    Integer(IntegerNotation),
    Float(FloatNotation),
}

impl Notation {
    pub const fn radix(self) -> u32 {
        match self {
            Notation::Integer(n) => n.radix(),
            Notation::Float(_) => FloatNotation::RADIX,
        }
    }
}

impl From<IntegerNotation> for Notation {
    fn from(v: IntegerNotation) -> Self {
        Notation::Integer(v)
    }
}

impl From<FloatNotation> for Notation {
    fn from(v: FloatNotation) -> Self {
        Notation::Float(v)
    }
}

/// Notation marker for floating point
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatNotation {
    Decimal,
    Scientific,
}

impl FloatNotation {
    pub const RADIX: u32 = 10;
}

/// Notation marker for Integers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerNotation {
    Bin,
    Decimal,
    Octal,
    Hex,
}

impl IntegerNotation {
    pub const fn radix(self) -> u32 {
        match self {
            IntegerNotation::Decimal => 10,
            IntegerNotation::Bin => 2,
            IntegerNotation::Octal => 8,
            IntegerNotation::Hex => 16,
        }
    }
}
