//! # document
//!
//! Provides static documentation tables and the [`Document`] type for hover content.
//!
//! Four tables are exposed as `pub static` slices:
//!
//! | Constant              | Indexed by                  | Length |
//! |-----------------------|-----------------------------|--------|
//! | [`KEYWORD_DOCS`]      | `Keyword as usize`          | 15     |
//! | [`BUILTIN_TYPE_DOCS`] | `BuiltinTypeKind as usize`  | 27     |
//! | [`FUNC_DOCS`]         | `FuncKind as usize`         | 7      |
//! | [`DIRECTIVE_DOCS`]    | preloaded [`InternedId`]     | 6      |
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
        INTERNED_IGNORE, INTERNED_OCTAL, INTERNED_SCIENT, INTERNED_WARN,
    },
};
use compilation::semantic::hir::hir_symbols::FuncKind;
use lang::keywords::Keyword;
use lang::types::builtins::BuiltinTypeKind;

/// 60 Dashes
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
    pub const fn func_docs(kind: FuncKind) -> &'static Document {
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
        description: "Defines a struct in the `nest->` section",
        example: Some(
            "Fields have a name and type. Commas between fields are optional.\n\n**Example:**\n```chrn\nnest->\nstruct Person {\n    name: str\n    age: u8\n}\n```",
        ),
    },
    Document {
        key: "enum",
        description: "Defines an enum in the `nest->` section",
        example: Some(
            "Variants may be untyped or carry a type. Commas between variants are optional.\n\n**Example:**\n```chrn\nnest->\nenum Status {\n    Pending\n    Active: Tuple<i32>\n    Completed\n}\n```",
        ),
    },
    Document {
        key: "import",
        description: "Loads another `.chrn` module so its exported symbols can be used",
        example: Some(
            "Use `as` to choose the local module name and `::` to access an export.\n\n**Direct import:**\n```chrn\nimport \"definitions.chrn\"\nvar->\n    item: definitions::Item\n```\n\n**Aliased import:**\n```chrn\nimport \"definitions.chrn\" as defs\nvar->\n    item: defs::Item\n```",
        ),
    },
    Document {
        key: "export",
        description: "Lets imported modules use a definition",
        example: Some(
            "Use `export` on a `let`, `alias`, `struct`, or `enum`. Structs and enums still belong in `nest->`.\n\n**Values and aliases:**\n```chrn\nexport let LIMIT = 42\nexport alias NonEmpty() = [!IsEmpty]\n```\n\n**Structs and enums:**\n```chrn\nnest->\nexport struct Thing { value: i32 }\nexport enum State { Ready Pending }\n```",
        ),
    },
    Document {
        key: "bind",
        description: "Links this config to an external data file",
        example: Some(
            "Use `bind` in a standalone config. You do not need it when the config is inside the data file between `@def` and `@end`.\n\n**Standalone config:**\n```chrn\nbind \"data.json\"\n```",
        ),
    },
    Document {
        key: "alias",
        description: "Declares a reusable group of predicates and directives",
        example: Some(
            "Give each parameter a type or a boundary such as `UnsignedInteger`. Use the alias in a condition block.\n\n**Reusable conditions:**\n```chrn\nalias NonBlank() = [!IsEmpty, !IsWhitespace]\nvar->\n    name: str [NonBlank()]\n```\n\n**Alias with a directive:**\n```chrn\nalias WarnIfEmpty() = [IsEmpty] #warn\nvar->\n    tag: str [WarnIfEmpty()]\n```",
        ),
    },
    Document {
        key: "let",
        description: "Declares a reusable value; the type is inferred by default",
        example: Some(
            "Declare a top-level value in the neutral section, or use `let … in …` to bind a value inside an expression.\n\n**Reusable declarations:**\n```chrn\nlet base = 10\nlet doubled = base * 2\n```\n\n**Scoped expression:**\n```chrn\nlet doubled = let value = 10 in value * 2\n```",
        ),
    },
    Document {
        key: "change",
        description: "Maps one or more chrn types to a language-specific type inside an override",
        example: Some(
            "Put `change` inside a built-in override group such as `types`. List the chrn types on the left and the language type to use on the right.\n\n**One chrn type:**\n```chrn\noverride JAVA=>types {\n    change bool = java::boolean\n}\n```\n\n**Several chrn types:**\n```chrn\noverride JAVA=>types {\n    change i8, i16 = java::int\n}\n```",
        ),
    },
    Document {
        key: "as",
        description: "Assigns a local module name to an import",
        example: Some(
            "```chrn\nimport \"module.chrn\" as mod\nlet x = mod::MAGIC_NUM - 2\nvar->\n    field: mod::EXTERN_TYPE\n```",
        ),
    },
    Document {
        key: "var->",
        description: "Starts the section describing top-level serialized data",
        example: Some(
            "Each entry has a name and type. Conditions check incoming data. Directives change how errors or output are handled. Names in `var->` can refer to definitions in `nest->` and the neutral section.\n\n**Fields, a condition, and a directive:**\n```chrn\nvar->\n    name: str [!IsEmpty]\n    separator: str [IsWhitespace] #warn\n```",
        ),
    },
    Document {
        key: "nest->",
        description: "Starts the section where structs and enums are defined",
        example: Some(
            "Types declared here can be referenced from `var->` and other nested types. `nest->` can search `var->`, itself, and the neutral section.\n\n**Nested types used by top-level data:**\n```chrn\nvar->\n    address: Address\nnest->\n    struct Address { city: str zip: u32 }\n    enum Color { Red Green Blue }\n```",
        ),
    },
    Document {
        key: "complex->",
        description: "Starts the section for type settings and language overrides",
        example: Some(
            "Use `for` to change the settings for a type. Use `override` to change built-in defaults for a language. Normal config blocks can nest two levels deep. Built-in override paths can go deeper.\n\n**Type settings:**\n```chrn\nnest->\nstruct Person { name: str age: u8 }\ncomplex->\nfor Person {\n    cases = [\"snake_case\", \"UpperSnakeCase\"]\n    age { default_val = 0 }\n}\n```\n\n**Language override:**\n```chrn\ncomplex->\noverride JAVA=>types {\n    change i8, i16 = java::int\n}\n```",
        ),
    },
    Document {
        key: "override",
        description: "Changes language defaults for everything or for one `for` block",
        example: Some(
            "At the root of `complex->`, `override LANGUAGE` changes the defaults everywhere that language is used. Inside a `for` block, it changes only that type or member. The inside override has priority over the root override.\n\n**Root override:**\n```chrn\ncomplex->\noverride JAVA {\n    types {\n        change i8, i16 = java::int\n    }\n}\n```\n\n**Override inside `for`:**\n```chrn\ncomplex->\nfor Person {\n    age {\n        override RUST=>types {\n            change u8 = rust::u32\n        }\n    }\n}\n```",
        ),
    },
    Document {
        key: "in",
        description: "Separates a scoped `let` value from the expression that uses it",
        example: Some(
            "```chrn\n@def\n\tlet result = let x = 10 in x * 2\n\t// result = 20\n@end\n```",
        ),
    },
    Document {
        key: "for",
        description: "Chooses the type to configure in `complex->`",
        example: Some(
            "Add `var` or `nest` before the type name when both sections contain that name. Put a member's settings inside a block named after the member.\n\n**Type and member settings:**\n```chrn\ncomplex->\nfor Person {\n    idents = \"Human\"\n    age { default_val = 0 }\n}\n```\n\n**Choose the `nest->` type:**\n```chrn\ncomplex->\nfor nest Person { idents = \"Human\" }\n```",
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
        description: "Boolean type",
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
        description: "Leaves the value's type to be found at runtime",
        example: Some(
            "Use `Runtime` when the data may hold different types and the config cannot name one type ahead of time.\n\n**Example:**\n```chrn\nvar->\n    payload: Runtime\n```",
        ),
    },
];

// ── Intrinsic functions ───────────────────────────────────────────────────────
//
// Indexed by `FuncKind as usize`.  Variants must appear in the same order as the
// `FuncKind` enum definition in `compilation::semantic::hir`.
/// Hover documentation for each Chern intrinsic (built-in) function.
///
/// Indexed by [`FuncKind`] discriminant via [`Document::func_docs`].
pub static FUNC_DOCS: [Document; 7] = [
    Document {
        key: "IsEmpty",
        description: "Checks whether a string or collection has length zero",
        example: Some(
            "Negate the predicate when the value must contain something.\n\n**Example:**\n```chrn\nvar->\n    items: List<str> [!IsEmpty]\n```",
        ),
    },
    Document {
        key: "IsWhitespace",
        description: "Checks whether a string contains only Unicode whitespace",
        example: Some("```chrn\nvar->\n    separator: str [IsWhitespace]\n```"),
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
pub static DIRECTIVE_DOCS: [Document; 6] = [
    Document {
        key: "warn",
        description: "Reports a failed serialized-data constraint as a warning instead of an error",
        example: Some(
            "Add it to the field whose failed condition should produce a warning.\n\n**Example:**\n```chrn\nvar->\n    separator: str [IsWhitespace] #warn\n```",
        ),
    },
    Document {
        key: "ignore",
        description: "Ignores data errors for the field it is added to",
        example: Some("```chrn\nvar->\n    payload: Runtime #ignore\n```"),
    },
    Document {
        key: "scient",
        description: "Outputs numeric values in scientific notation",
        example: Some("```chrn\nvar->\n\tpi: f64 #scient\n```"),
    },
    Document {
        key: "hex",
        description: "Outputs numeric values in hexadecimal notation",
        example: Some("```chrn\nvar->\n    color: u32 #hex\n```"),
    },
    Document {
        key: "bin",
        description: "Outputs numeric values in binary notation",
        example: Some("```chrn\nvar->\n\tflags: u8 #bin\n```"),
    },
    Document {
        key: "octal",
        description: "Outputs numeric values in octal notation",
        example: Some("```chrn\nvar->\n\tperm: u32 #octal\n```"),
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
        description: "Lets serialized names use more than one letter case style",
        example: Some(
            "Assign one convention directly or a list of conventions. The option applies to the configured type or member.\n\n**Several accepted conventions:**\n```chrn\ncomplex->\nfor Person {\n    cases = [\"snake_case\", \"UpperSnakeCase\"]\n}\n```",
        ),
    },
    Document {
        key: "idents",
        description: "Adds other names that can match the serialized value",
        example: Some(
            "Use it on a root to rename the type during matching, or in a member block to rename that member. A single string does not require brackets.\n\n**Type identifier:**\n```chrn\ncomplex->\nfor Person { idents = \"Human\" }\n```\n\n**Member identifier:**\n```chrn\ncomplex->\nfor Person {\n    name { idents = [\"display_name\", \"full_name\"] }\n}\n```",
        ),
    },
    Document {
        key: "default_val",
        description: "Supplies a member value when serialized data is absent",
        example: Some(
            "Set it in the member's config block. The value must match the member's type.\n\n**Example:**\n```chrn\ncomplex->\nfor Person {\n    age { default_val = 0 }\n}\n```",
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
            "FUNC_DOCS must have one entry per FuncKind variant"
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
            2,
            "each override form should have its own chrn example"
        );
        assert_eq!(
            markdown.matches("```").count(),
            4,
            "both override examples should have closed Markdown fences"
        );
    }
}
