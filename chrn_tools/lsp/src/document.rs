//! # document
//!
//! Provides static documentation tables and the [`Document`] type for hover content.
//!
//! Five tables are exposed as `pub static` arrays:
//!
//! | Constant              | Indexed by                  | Length |
//! |-----------------------|-----------------------------|--------|
//! | [`KEYWORD_DOCS`]      | `Keyword as usize`          | 15     |
//! | [`BUILTIN_TYPE_DOCS`] | `BuiltinTypeKind as usize`  | 27     |
//! | [`FUNC_DOCS`]         | `BuiltinFuncKind as usize`  | 7      |
//! | [`DIRECTIVE_DOCS`]    | preloaded [`InternedId`]     | 7      |
//! | [`CONFIG_OPTION_DOCS`]| preloaded [`InternedId`]     | 3      |
//!
//! ## Alignment invariant
//!
//! **The entries in each enum-indexed table MUST remain aligned with the discriminant
//! values of their respective enum.**  Adding a new keyword, builtin type, or intrinsic
//! function requires inserting the corresponding [`Document`] entry at the correct index
//! and updating the length in this comment.
//!
//! Accessor methods on [`Document`] (`keyword_docs`, `builtin_type_docs`, `func_docs`)
//! index directly into these arrays; an out-of-bounds index will panic at runtime.

use chrn_utils::{
    id_types::InternedId,
    intern::{
        INTERNED_BIN, INTERNED_CASES, INTERNED_DEFAULT_VAL, INTERNED_HEX, INTERNED_IDENTS,
        INTERNED_IGNORE, INTERNED_MAX_NUMERIC_BITS, INTERNED_OCTAL, INTERNED_SCIENT, INTERNED_WARN,
    },
};
use compilation::semantic::hir::hir_symbols::BuiltinFuncKind;
use lang::keywords::Keyword;
use lang::types::builtins::BuiltinTypeKind;

/// Separator between a hover description and its supporting Markdown.
pub static HOVER_DASHES: &str = "------------------------------------------------------------";

/// A structured documentation entry for a language construct.
/// Separates the name and description from the rendered presentation,
/// enabling both direct lookup by key and formatted hover output.
pub struct Document {
    /// The name of the construct (e.g. "struct", "BigInt", "i32")
    pub key: &'static str,
    /// A brief description of the construct
    pub description: &'static str,
    /// Optional supporting Markdown shown in hover popups. A bare fenced code
    /// block is rendered under an `Example` heading; richer entries may supply
    /// prose and multiple labeled examples directly.
    pub example: Option<&'static str>,
}

impl Document {
    /// Composes a markdown hover string from the document's fields.
    pub fn compose(&self) -> String {
        let header = format!("**{}** — {}", self.key, self.description);
        match self.example {
            Some(example) if example.trim_start().starts_with("```") => format!(
                "{}\n\n{}\n\n**Example:**\n{}",
                header, HOVER_DASHES, example
            ),
            Some(supporting_markdown) => {
                format!("{}\n\n{}\n\n{}", header, HOVER_DASHES, supporting_markdown)
            }
            None => header,
        }
    }

    /// Returns the document for a given keyword variant.
    pub const fn keyword_docs(kw: Keyword) -> &'static Document {
        &KEYWORD_DOCS[kw as usize]
    }

    /// Returns the document for a given builtin type kind.
    pub const fn builtin_type_docs(kind: BuiltinTypeKind) -> &'static Document {
        &BUILTIN_TYPE_DOCS[kind as usize]
    }

    /// Returns the document for a given intrinsic function kind.
    pub const fn func_docs(kind: BuiltinFuncKind) -> &'static Document {
        &FUNC_DOCS[kind as usize]
    }

    /// Returns the document for a directive by interned id, or `None` if unknown.
    pub const fn directive_docs(name_id: InternedId) -> Option<&'static Document> {
        match name_id.id {
            INTERNED_WARN => Some(&DIRECTIVE_DOCS[0]),
            INTERNED_IGNORE => Some(&DIRECTIVE_DOCS[1]),
            INTERNED_SCIENT => Some(&DIRECTIVE_DOCS[2]),
            INTERNED_HEX => Some(&DIRECTIVE_DOCS[3]),
            INTERNED_BIN => Some(&DIRECTIVE_DOCS[4]),
            INTERNED_OCTAL => Some(&DIRECTIVE_DOCS[5]),
            INTERNED_MAX_NUMERIC_BITS => Some(&DIRECTIVE_DOCS[6]),
            _ => None,
        }
    }

    /// Returns the document for a config option by interned id, or `None` if unknown.
    pub const fn config_option_docs(name_id: InternedId) -> Option<&'static Document> {
        match name_id.id {
            INTERNED_CASES => Some(&CONFIG_OPTION_DOCS[0]),
            INTERNED_IDENTS => Some(&CONFIG_OPTION_DOCS[1]),
            INTERNED_DEFAULT_VAL => Some(&CONFIG_OPTION_DOCS[2]),
            _ => None,
        }
    }
}

//  Keywords

// ── Keywords ─────────────────────────────────────────────────────────────────
//
// Indexed by `Keyword as usize`.  Variants must appear in the same order as the
// `Keyword` enum definition in `lang::keywords`.
//FIX: Key should be the interned id, or maybe a mixture of both depending on DECISIONS
/// Hover documentation for each Chern language keyword.
///
/// Indexed by [`Keyword`] discriminant via [`Document::keyword_docs`].
pub static KEYWORD_DOCS: [Document; 15] = [
    Document {
        key: "struct",
        description: "Declares a type with named fields in `nest->`",
        example: Some(
            r#"Fields require a type. Commas between fields are optional.

**Fields and nested types:**
```chrn
nest->
struct Address { city: str }
struct Person {
    name: str
    address: Address
}
```

**Field conditions and directives (condition and whole-type analysis unfinished):**
```chrn
nest->
struct Person {
    name: str [!IsEmpty]
    age: u8 #hex
} #warn
```
A directive after the closing brace applies to the whole struct."#,
        ),
    },
    Document {
        key: "enum",
        description: "Declares a type with named variants in `nest->`",
        example: Some(
            r#"Variants may have a type or no associated data. Commas between variants are optional.

**Variants with and without data:**
```chrn
nest->
enum Color {
    Default
    Named: str
    Rgb: Tuple<u8, u8, u8>
}
```

**Variant and enum directives (whole-type analysis unfinished):**
```chrn
nest->
enum Status {
    Pending
    Complete: u32 #hex
} #warn
```
A directive after the closing brace applies to the whole enum."#,
        ),
    },
    Document {
        key: "import",
        description: "Imports a `.chrn` module so its exported declarations can be used",
        example: Some(
            r#"Both examples use this file. Relative import paths are resolved from the server's working directory; place `definitions.chrn` there for these examples.

**`definitions.chrn`:**
```chrn
export let LIMIT = 10
nest->
export struct Item { name: str }
```

**Direct import — `main.chrn`:**
```chrn
import "definitions.chrn"
let limit = definitions::LIMIT
var->
    item: definitions::Item
```
Without `as`, the file name before its first extension is the local module name.

**Aliased import — `main.chrn`:**
```chrn
import "definitions.chrn" as defs
let limit = defs::LIMIT
var->
    item: defs::Item
```
Use `::` to access an exported declaration. `as` changes the local module name."#,
        ),
    },
    Document {
        key: "export",
        description: "Makes a declaration available to modules that import this file",
        example: Some(
            r#"Use `export` on `let`, `alias`, `struct`, or `enum`. It does not change where the declaration is allowed.

**Values and aliases:**
Alias constraint analysis is unfinished.
```chrn
export let LIMIT = 42
export alias NonEmpty() = [!IsEmpty]
```

**Structs and enums:**
```chrn
nest->
export struct Item { name: str }
export enum Status { Ready Pending }
```
Import the file and use `module_name::declaration_name` to access an export."#,
        ),
    },
    Document {
        key: "bind",
        description: "Associates a standalone chrn configuration with a serialized-data file",
        example: Some(
            r#"Declare `bind` in the neutral section, before any section keyword. The serialized-data file must exist. Relative paths are resolved from the working directory.

**Standalone configuration:**
```chrn
bind "data.json"
var->
    name: str
```

**Chrn embedding:**
```chrn
@def
var->
    name: str
@end
{"name": "Ada"}
```
An embedding describes the serialized data after `@end` and does not need `bind`."#,
        ),
    },
    Document {
        key: "alias",
        description: "Names a reusable group of conditions, directives, or both",
        example: Some(
            r#"Alias constraint analysis is unfinished, and calls with arguments are currently rejected by the parser. The following examples illustrate the language syntax, not working constraint checks.

Declare aliases in the neutral section and call them inside a condition block. Parameters require a concrete type or a type boundary.

**Reusable predicates:**
```chrn
alias NonBlank() = [!IsEmpty, !IsWhitespace]
var->
    name: str [NonBlank()]
```

**Conditions with a directive:**
```chrn
alias WarnIfEmpty() = [!IsEmpty] #warn
var->
    tag: str [WarnIfEmpty()]
```
The condition requires a nonempty value; `#warn` selects a warning if it fails.

**Directive only:**
```chrn
alias WarnOnly() = #warn
var->
    name: str [WarnOnly()]
```

**Typed parameters:**
```chrn
alias Enabled(enabled: bool) = [enabled]
var->
    name: str [Enabled(true)]
```

**Boundary parameters:**
```chrn
alias Ordered(min: Integer, max: Integer) = [min < max]
var->
    name: str [Ordered(1, 10)]
```
The last two examples evaluate the arguments, not the serialized value."#,
        ),
    },
    Document {
        key: "let",
        description: "Binds a name to a value with an inferred type",
        example: Some(
            r#"Declare top-level values in the neutral section, before any section keyword.

**Values and expressions:**
```chrn
let base = 10
let doubled = base * 2
```
Local `let ... in ...` expressions are not implemented."#,
        ),
    },
    Document {
        key: "change",
        description: "Maps chrn types to an extern type inside a language override",
        example: Some(
            r#"Use `change` inside the override's `types` block. The right-hand side must be an extern type for that language.

**One source type:**
```chrn
complex->
override JAVA=>types {
    change bool = java::boolean
}
```

**Several source types:**
```chrn
complex->
override JAVA=>types {
    change i8, i16 = java::int
}
```

**Infer the current member's type:**
```chrn
nest->
struct Person { age: u8 }
complex->
for Person {
    age {
        override RUST=>types { change = rust::u32 }
    }
}
```
Omitting the left-hand type uses the configured member's type.

**Explicit member type:**
```chrn
nest->
struct Person { age: u8 }
complex->
for Person {
    age {
        override RUST=>types { change u8 = rust::u32 }
    }
}
```
In a member override, an explicit source type must match that member."#,
        ),
    },
    Document {
        key: "as",
        description: "Chooses the local name used to access an imported module",
        example: Some(
            r#"Both examples use this file. Relative import paths are resolved from the server's working directory; place `definitions.chrn` there for these examples.

**`definitions.chrn`:**
```chrn
export let LIMIT = 10
nest->
export struct Item { name: str }
```

**Direct import — `main.chrn`:**
```chrn
import "definitions.chrn"
let limit = definitions::LIMIT
var->
    item: definitions::Item
```
Without `as`, the file name before its first extension is the local module name.

**Aliased import — `main.chrn`:**
```chrn
import "definitions.chrn" as defs
let limit = defs::LIMIT
var->
    item: defs::Item
```
Use `::` to access an exported declaration. `as` changes the local module name."#,
        ),
    },
    Document {
        key: "var->",
        description: "Starts the section declaring front-facing serialized values",
        example: Some(
            r#"Each declaration has a name and a type. This section searches `nest->`, then the neutral section, for definitions.

**Values and structural types:**
```chrn
var->
    name: str
    address: Address
nest->
struct Address { city: str }
```

**Conditions and directives (condition analysis unfinished):**
```chrn
var->
    name: str [!IsEmpty]
    separator: str [IsWhitespace] #warn
```
Conditions constrain serialized data. Directives select diagnostic or serialization behavior."#,
        ),
    },
    Document {
        key: "nest->",
        description: "Starts the section declaring struct and enum types",
        example: Some(
            r#"Declarations here can refer to definitions in `var->`, `nest->`, and the neutral section, in that search order.

**Struct used by a front-facing value:**
```chrn
var->
    address: Address
nest->
struct Address { city: str zip: u32 }
```

**Enum used by a front-facing value:**
```chrn
var->
    color: Color
nest->
enum Color { Red Green Blue }
```"#,
        ),
    },
    Document {
        key: "complex->",
        description:
            "Starts the section configuring existing types, members, and language mappings",
        example: Some(
            r#"A `for` block selects a configuration target. Its option assignments configure that target; nested member blocks select individual fields or variants. `override` configures language-specific type mappings.

**Type and member options:**
```chrn
nest->
struct Person { name: str age: i32 }
complex->
for Person {
    cases = ["snake_case", "UpperSnakeCase"]
    age { default_val = 0 }
}
```
A normal config allows a root and one level of member blocks. Override blocks can nest more deeply.

**Language mappings:**
```chrn
complex->
override JAVA=>types {
    change i8, i16 = java::int
}
```
`=>types` shortens the nested form `{ types { ... } }`."#,
        ),
    },
    Document {
        key: "override",
        description: "Sets language-specific type mappings globally or within a type configuration",
        example: Some(
            r#"Use a language name such as `JAVA` or `RUST`, then a `types` block containing `change` assignments. An override inside `for` has priority over a global override for the same language.

**Root override:**
```chrn
complex->
override JAVA {
    types {
        change i8, i16 = java::int
    }
}
```
This sets mappings throughout the configuration for Java. `override JAVA=>types { ... }` is the shortened form.

**Override inside `for`:**
```chrn
nest->
struct Person { age: u8 score: i16 }
complex->
for Person {
    override JAVA=>types { change i16 = java::int }
}
```
This Java mapping is local to `Person`.

**Override inside a member:**
```chrn
nest->
struct Person { age: u8 }
complex->
for Person {
    age {
        override RUST=>types { change = rust::u32 }
    }
}
```
This Rust mapping is local to `age`."#,
        ),
    },
    Document {
        key: "in",
        description: "Reserved for local `let` expressions; not implemented",
        example: None,
    },
    Document {
        key: "for",
        description: "Selects an existing type or front-facing value to configure in `complex->`",
        example: Some(
            r#"Option assignments on the root configure the selected target. A block named after a member configures that member. These options do not rename chrn declarations.

**Type and member options:**
```chrn
nest->
struct Person { name: str age: i32 }
complex->
for Person {
    idents = "Human"
    age { default_val = 0 }
}
```

**Select a section explicitly:**
```chrn
var->
    Person: Person
nest->
struct Person { name: str }
complex->
for var Person {}
for nest Person { idents = "Human" }
```
`var` selects the front-facing declaration; `nest` selects the structural type. An empty block selects the target without assigning options.

**Shortened member block:**
```chrn
nest->
struct Person { name: str }
complex->
for Person=>name { idents = "display_name" }
```
`=>name` selects `name` without an extra pair of braces.

**Imported type:**
`definitions.chrn` exports the type:
```chrn
nest->
export struct Item { name: str }
```
`main.chrn` imports and configures it:
```chrn
import "definitions.chrn"
complex->
for definitions::Item { idents = "Product" }
```
The importing module's configuration takes priority over configuration for that type in the imported module. Relative imports use the server's working directory."#,
        ),
    },
];

// ── Builtin types ─────────────────────────────────────────────────────────────
//
// Indexed by `BuiltinTypeKind as usize`.  Variants must appear in the same order
// as the `BuiltinTypeKind` enum definition in `lang::types::builtins`.
/// Hover documentation for each Chern builtin type.
///
/// Indexed by [`BuiltinTypeKind`] discriminant via [`Document::builtin_type_docs`].
pub static BUILTIN_TYPE_DOCS: [Document; 27] = [
    Document {
        key: "i8",
        description: "8-bit signed integer",
        example: None,
    },
    Document {
        key: "u8",
        description: "8-bit unsigned integer",
        example: None,
    },
    Document {
        key: "i16",
        description: "16-bit signed integer",
        example: None,
    },
    Document {
        key: "u16",
        description: "16-bit unsigned integer",
        example: None,
    },
    Document {
        key: "f16",
        description: "16-bit floating point",
        example: None,
    },
    Document {
        key: "i32",
        description: "32-bit signed integer",
        example: None,
    },
    Document {
        key: "u32",
        description: "32-bit unsigned integer",
        example: None,
    },
    Document {
        key: "f32",
        description: "32-bit floating point",
        example: None,
    },
    Document {
        key: "i64",
        description: "64-bit signed integer",
        example: None,
    },
    Document {
        key: "u64",
        description: "64-bit unsigned integer",
        example: None,
    },
    Document {
        key: "f64",
        description: "64-bit floating point",
        example: None,
    },
    Document {
        key: "i128",
        description: "128-bit signed integer",
        example: None,
    },
    Document {
        key: "u128",
        description: "128-bit unsigned integer",
        example: None,
    },
    Document {
        key: "f128",
        description: "128-bit floating point",
        example: None,
    },
    Document {
        key: "sized",
        description: "Platform-sized signed integer",
        example: None,
    },
    Document {
        key: "unsized",
        description: "Platform-sized unsigned integer",
        example: None,
    },
    Document {
        key: "str",
        description: "UTF-8 encoded string",
        example: None,
    },
    Document {
        key: "char",
        description: "A single Unicode character",
        example: None,
    },
    Document {
        key: "nil",
        description: "Generic nil/null value that adapts to the target language when possible",
        example: None,
    },
    Document {
        key: "bool",
        description: "Boolean value (`true` or `false`)",
        example: None,
    },
    Document {
        key: "BigInt",
        description: "Integer represented as an unbounded string",
        example: None,
    },
    Document {
        key: "BigFloat",
        description: "Floating point type represented as an unbounded string",
        example: None,
    },
    Document {
        key: "List",
        description: "Ordered collection with one element type",
        example: Some("```chrn\nvar->\n    names: List<str>\n```"),
    },
    Document {
        key: "Set",
        description: "Collection of unique values with one element type",
        example: Some("```chrn\nvar->\n    tags: Set<str>\n```"),
    },
    Document {
        key: "Map",
        description: "Key-value collection with key and value types",
        example: Some("```chrn\nvar->\n    scores: Map<str, i32>\n```"),
    },
    Document {
        key: "Tuple",
        description: "Fixed-position collection with any number of element types",
        example: Some("```chrn\nvar->\n    coordinate: Tuple<f64, f64>\n```"),
    },
    Document {
        key: "Runtime",
        description: "Defers the value's type to runtime",
        example: Some(
            "Use `Runtime` when the serialized value's type is not known in advance.\n\n**Example:**\n```chrn\nvar->\n    payload: Runtime\n```",
        ),
    },
];

// ── Intrinsic functions ───────────────────────────────────────────────────────
//
// Indexed by `BuiltinFuncKind as usize`.  Variants must appear in the same order as the
// `BuiltinFuncKind` enum definition in `compilation::semantic::hir`.
/// Hover documentation for each Chern intrinsic (built-in) function.
///
/// Indexed by [`BuiltinFuncKind`] discriminant via [`Document::func_docs`].
pub static FUNC_DOCS: [Document; 7] = [
    Document {
        key: "IsEmpty",
        description: "Checks whether a string or collection has length zero",
        example: Some(
            r#"Condition analysis is unfinished. Use this predicate without parentheses. Prefix it with `!` to require a nonempty value.

**String:**
```chrn
var->
    placeholder: str [IsEmpty]
    name: str [!IsEmpty]
```

**Collections:**
```chrn
var->
    names: List<str> [!IsEmpty]
    tags: Set<str> [!IsEmpty]
    scores: Map<str, i32> [!IsEmpty]
    coordinate: Tuple<i32, i32> [!IsEmpty]
```"#,
        ),
    },
    Document {
        key: "IsWhitespace",
        description: "Checks whether a string contains only Unicode whitespace",
        example: Some(
            r#"Condition analysis is unfinished. Use this predicate without parentheses. Negating it requires a string that is not entirely whitespace; it does not forbid whitespace between other characters.

**Whitespace-only string:**
```chrn
var->
    separator: str [IsWhitespace]
```

**Nonblank string:**
```chrn
var->
    name: str [!IsEmpty, !IsWhitespace]
```"#,
        ),
    },
    Document {
        key: "Contains",
        description: "Planned predicate for checking whether a value contains an argument; not implemented",
        example: Some(
            "This is planned syntax and does not work yet.\n\n**Planned syntax:**\n```chrn\nvar->\n    project: str [Contains(\"chrn\")]\n```",
        ),
    },
    Document {
        key: "Range",
        description: "Planned predicate for checking a value or length against a range; not implemented",
        example: Some(
            "This is planned syntax and does not work yet. The first number will be included and the second will not. Numbers will use their value; strings and collections will use their length.\n\n**Planned number check:**\n```chrn\nvar->\n    percentage: f64 [Range(0.0, 100.0)]\n```\n\n**Planned string length check:**\n```chrn\nvar->\n    username: str [Range(1, 25)]\n```",
        ),
    },
    Document {
        key: "StartsW",
        description: "Planned predicate for checking a value's prefix; not implemented",
        example: Some(
            "This is planned syntax and does not work yet.\n\n**Planned syntax:**\n```chrn\nvar->\n    resource: str [StartsW(\"chrn:\")]\n```",
        ),
    },
    Document {
        key: "EndsW",
        description: "Planned predicate for checking a value's suffix; not implemented",
        example: Some(
            "This is planned syntax and does not work yet.\n\n**Planned syntax:**\n```chrn\nvar->\n    config_file: str [EndsW(\".chrn\")]\n```",
        ),
    },
    Document {
        key: "Equals",
        description: "Planned predicate for equality with an argument; not implemented",
        example: Some(
            "This is planned syntax and does not work yet.\n\n**Planned syntax:**\n```chrn\nlet required_version = 2\nvar->\n    version: u32 [Equals(required_version)]\n```",
        ),
    },
];

// ── Directives ────────────────────────────────────────────────────────────────
//
// The accessor explicitly maps each preloaded id to its local table index.
/// Hover documentation for each Chern directive.
///
/// Indexed by preloaded [`InternedId`] via [`Document::directive_docs`].
pub static DIRECTIVE_DOCS: [Document; 7] = [
    Document {
        key: "warn",
        description: "Selects a warning instead of an error for failed serialized-data constraints",
        example: Some(
            r#"**One value:**
```chrn
var->
    name: str [!IsEmpty] #warn
```

**Whole struct (analysis unfinished):**
```chrn
nest->
struct Pair { first: str second: str } #warn
```
Put the directive after the closing brace to apply it throughout the declared type.

**Whole enum (analysis unfinished):**
```chrn
nest->
enum Choice { First: str Second: str } #warn
```

**Reusable directive (alias analysis unfinished):**
```chrn
alias ApplyDirective() = #warn
var->
    name: str [!IsEmpty, ApplyDirective()]
```"#,
        ),
    },
    Document {
        key: "ignore",
        description: "Suppresses serialized-data errors for the value or type it is applied to",
        example: Some(
            r#"**One value:**
```chrn
var->
    payload: Runtime #ignore
```

**Whole struct (analysis unfinished):**
```chrn
nest->
struct Pair { first: str second: str } #ignore
```
Put the directive after the closing brace to apply it throughout the declared type.

**Whole enum (analysis unfinished):**
```chrn
nest->
enum Choice { First: str Second: str } #ignore
```

**Reusable directive (alias analysis unfinished):**
```chrn
alias ApplyDirective() = #ignore
var->
    payload: Runtime [ApplyDirective()]
```"#,
        ),
    },
    Document {
        key: "scient",
        description: "Selects scientific notation for serialized numeric output",
        example: Some(
            r#"**One value:**
```chrn
var->
    amount: f64 #scient
```

**Whole struct (analysis unfinished):**
```chrn
nest->
struct Pair { first: f64 second: f64 } #scient
```
Put the directive after the closing brace to apply it throughout the declared type.

**Whole enum (analysis unfinished):**
```chrn
nest->
enum Choice { First: f64 Second: f64 } #scient
```

**Reusable directive (alias analysis unfinished):**
```chrn
alias ApplyDirective() = #scient
var->
    amount: f64 [ApplyDirective()]
```"#,
        ),
    },
    Document {
        key: "hex",
        description: "Selects hexadecimal notation for serialized numeric output",
        example: Some(
            r#"**One value:**
```chrn
var->
    color: u32 #hex
```

**Whole struct (analysis unfinished):**
```chrn
nest->
struct Pair { first: u32 second: u32 } #hex
```
Put the directive after the closing brace to apply it throughout the declared type.

**Whole enum (analysis unfinished):**
```chrn
nest->
enum Choice { First: u32 Second: u32 } #hex
```

**Reusable directive (alias analysis unfinished):**
```chrn
alias ApplyDirective() = #hex
var->
    color: u32 [ApplyDirective()]
```"#,
        ),
    },
    Document {
        key: "bin",
        description: "Selects binary notation for serialized numeric output",
        example: Some(
            r#"**One value:**
```chrn
var->
    flags: u8 #bin
```

**Whole struct (analysis unfinished):**
```chrn
nest->
struct Pair { first: u8 second: u8 } #bin
```
Put the directive after the closing brace to apply it throughout the declared type.

**Whole enum (analysis unfinished):**
```chrn
nest->
enum Choice { First: u8 Second: u8 } #bin
```

**Reusable directive (alias analysis unfinished):**
```chrn
alias ApplyDirective() = #bin
var->
    flags: u8 [ApplyDirective()]
```"#,
        ),
    },
    Document {
        key: "octal",
        description: "Selects octal notation for serialized numeric output",
        example: Some(
            r#"**One value:**
```chrn
var->
    permissions: u32 #octal
```

**Whole struct (analysis unfinished):**
```chrn
nest->
struct Pair { first: u32 second: u32 } #octal
```
Put the directive after the closing brace to apply it throughout the declared type.

**Whole enum (analysis unfinished):**
```chrn
nest->
enum Choice { First: u32 Second: u32 } #octal
```

**Reusable directive (alias analysis unfinished):**
```chrn
alias ApplyDirective() = #octal
var->
    permissions: u32 [ApplyDirective()]
```"#,
        ),
    },
    Document {
        key: "max_numeric_bits",
        description: "Sets the compiler's numeric bit limit",
        example: Some(
            r#"Limits numeric literals and computed numeric values. Accepts one integer literal representable as `u32`, not a `let` name or an arithmetic expression.

The compiler needs the limit before evaluating any expressions. If setting the limit required evaluation, a value larger than that limit could be evaluated first. A literal lets the compiler read the setting before evaluation begins.

Only the entry module can change this setting; imported modules use the entry module's limit.

**Within an 8-bit limit:**
```chrn
#max_numeric_bits(8)
let value = 255
let product = 15 * 17
```
Both values are `255`, whose magnitude fits in eight bits.

**Literal exceeds the limit (error):**
```chrn
#max_numeric_bits(8)
let value = 256
```
`256` needs nine bits, so the compiler rejects it.

**Computed result exceeds the limit (error):**
```chrn
#max_numeric_bits(8)
let product = 16 * 16
```
Each operand fits in eight bits, but the result is `256` and needs nine. The compiler rejects the multiplication."#,
        ),
    },
];

// ── Config Options ────────────────────────────────────────────────────────────
//
/// Hover documentation for schema config options used in `complex->`.
///
/// Indexed by preloaded [`InternedId`] via [`Document::config_option_docs`].
pub static CONFIG_OPTION_DOCS: [Document; 3] = [
    Document {
        key: "cases",
        description: "Sets letter-case conventions accepted for serialized identifiers",
        example: Some(
            r#"Use this option on a struct, enum, or member. Assign one convention as a string or several as a list. The chrn declaration keeps its original name.

**One convention on a type:**
```chrn
nest->
struct Person { displayName: str }
complex->
for Person { cases = "snake_case" }
```

**Several conventions on a type:**
```chrn
nest->
enum Status { InProgress Complete }
complex->
for Status { cases = ["snake_case", "UpperSnakeCase"] }
```

**Member convention:**
```chrn
nest->
struct Person { displayName: str }
complex->
for Person {
    displayName { cases = "snake_case" }
}
```"#,
        ),
    },
    Document {
        key: "idents",
        description: "Sets alternative identifiers accepted from serialized data",
        example: Some(
            r#"Use a string for one identifier or a list for several. On a config root, this option names the configured type; on a config member, it names the member, not the member's type. It does not rename either chrn declaration.

**Type identifier:**
```chrn
nest->
struct Person { name: str }
complex->
for Person { idents = "Human" }
```

**Field identifiers:**
```chrn
nest->
struct Person { name: str }
complex->
for Person {
    name { idents = ["display_name", "full_name"] }
}
```

**Enum and variant identifiers:**
```chrn
nest->
enum Status { Ready Pending }
complex->
for Status {
    idents = ["State", "Status"]
    Ready { idents = "ready" }
}
```"#,
        ),
    },
    Document {
        key: "default_val",
        description:
            "Sets a member default for null/None data where the target language supports it",
        example: Some(
            r#"Set this option in a member's config block. The value must match the member's type. This is serialization metadata; it does not initialize a chrn variable.

**Literal default:**
```chrn
nest->
struct Person { age: i32 }
complex->
for Person {
    age { default_val = 0 }
}
```

**Default from a named value:**
```chrn
let default_name = "Guest"
nest->
struct Person { name: str }
complex->
for Person {
    name { default_val = default_name }
}
```"#,
        ),
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_docs_len() {
        assert_eq!(
            KEYWORD_DOCS.len(),
            15,
            "KEYWORD_DOCS must have one entry per Keyword variant"
        );
    }

    #[test]
    fn test_builtin_type_docs_len() {
        assert_eq!(
            BUILTIN_TYPE_DOCS.len(),
            27,
            "BUILTIN_TYPE_DOCS must have one entry per BuiltinTypeKind variant"
        );
    }

    #[test]
    fn test_func_docs_len() {
        assert_eq!(
            FUNC_DOCS.len(),
            7,
            "FUNC_DOCS must have one entry per BuiltinFuncKind variant"
        );
    }

    #[test]
    fn directive_docs_map_preloaded_ids_to_their_documents() {
        let expected = [
            (INTERNED_WARN, "warn"),
            (INTERNED_IGNORE, "ignore"),
            (INTERNED_SCIENT, "scient"),
            (INTERNED_HEX, "hex"),
            (INTERNED_BIN, "bin"),
            (INTERNED_OCTAL, "octal"),
            (INTERNED_MAX_NUMERIC_BITS, "max_numeric_bits"),
        ];

        for (id, key) in expected {
            assert_eq!(
                Document::directive_docs(InternedId::new(id)).map(|document| document.key),
                Some(key),
                "directive id {id} should resolve to {key} documentation"
            );
        }
    }

    #[test]
    fn config_option_docs_map_preloaded_ids_to_their_documents() {
        let expected = [
            (INTERNED_CASES, "cases"),
            (INTERNED_IDENTS, "idents"),
            (INTERNED_DEFAULT_VAL, "default_val"),
        ];

        for (id, key) in expected {
            assert_eq!(
                Document::config_option_docs(InternedId::new(id)).map(|document| document.key),
                Some(key),
                "config option id {id} should resolve to {key} documentation"
            );
        }
    }

    #[test]
    fn interned_document_lookups_reject_unknown_ids() {
        assert!(Document::directive_docs(InternedId::new(INTERNED_CASES)).is_none());
        assert!(Document::directive_docs(InternedId::new(u32::MAX)).is_none());

        assert!(Document::config_option_docs(InternedId::new(INTERNED_WARN)).is_none());
        assert!(Document::config_option_docs(InternedId::new(u32::MAX)).is_none());
    }

    #[test]
    fn compose_labels_a_single_fenced_example() {
        let document = Document {
            key: "sample",
            description: "Demonstrates the compact documentation form",
            example: Some("```chrn\nlet answer = 42\n```"),
        };

        assert_eq!(
            document.compose(),
            format!(
                "**sample** — Demonstrates the compact documentation form\n\n{HOVER_DASHES}\n\n**Example:**\n```chrn\nlet answer = 42\n```"
            )
        );
    }

    #[test]
    fn compose_preserves_rich_markdown_without_an_extra_example_label() {
        let rich_markdown = "Use the first form at a config root.\n\n**Root form:**\n```chrn\noverride JAVA {}\n```\n\nUse the second form inside a type config.\n\n**Embedded form:**\n```chrn\nfor Person { override RUST {} }\n```";
        let document = Document {
            key: "sample",
            description: "Demonstrates multiple forms",
            example: Some(rich_markdown),
        };

        assert_eq!(
            document.compose(),
            format!(
                "**sample** — Demonstrates multiple forms\n\n{HOVER_DASHES}\n\n{rich_markdown}"
            )
        );
        assert!(!document.compose().contains("**Example:**"));
    }

    #[test]
    fn override_docs_explain_global_and_embedded_forms() {
        let markdown = Document::keyword_docs(Keyword::Override).compose();

        let root_heading = markdown
            .find("Root override")
            .expect("override documentation should identify the root form");
        let inside_for_heading = markdown
            .find("Override inside `for`")
            .expect("override documentation should identify the form inside `for`");

        assert!(
            root_heading < inside_for_heading,
            "the root form should be introduced before the form inside `for`"
        );
        assert!(
            markdown[root_heading..inside_for_heading].contains("override JAVA"),
            "the root section should show an override config root"
        );
        assert!(
            markdown[inside_for_heading..].contains("for Person"),
            "the inside section should show the surrounding type config"
        );
        assert!(
            markdown[inside_for_heading..].contains("override RUST"),
            "the inside section should show a local language override"
        );
        assert!(
            markdown.contains("priority"),
            "the documentation should explain embedded precedence over a global override"
        );
        assert_eq!(
            markdown.matches("```chrn").count(),
            3,
            "each override form should have its own chrn example"
        );
        assert_eq!(
            markdown.matches("```").count(),
            6,
            "all override examples should have closed Markdown fences"
        );
    }
}
