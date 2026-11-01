use crate::keywords::{KEYWORDS_ARRAY, Keyword};
use chrn_utils::intern::Intern;

// Generate the cases and exhaustive match together so new variants require regression coverage.
macro_rules! keyword_cases {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        #[test]
        fn keyword_interned_ids_round_trip_exhaustively() {
            let mut interner = Intern::init();
            let cases = [$(Keyword::$variant),+];
            let mut expected_names = Vec::new();

            for keyword in cases {
                let expected_name = match keyword {
                    $(Keyword::$variant => $name),+
                };
                let name_id = keyword.name_id();
                assert_eq!(interner.extract_utf8_str(name_id), expected_name, "{keyword:?}");
                assert_eq!(interner.intern_str(expected_name), name_id, "{keyword:?}");
                assert_eq!(
                    Keyword::try_from_interned_id(name_id),
                    Some(keyword),
                    "missing or incorrect reverse conversion for {keyword:?}",
                );
                expected_names.push(expected_name);
            }

            let mut keyword_names = KEYWORDS_ARRAY.to_vec();
            keyword_names.sort_unstable();
            expected_names.sort_unstable();
            assert_eq!(keyword_names, expected_names);

            for name in ["self", "true", "false", "i32", "ordinary_identifier", "Virtual"] {
                assert_eq!(
                    Keyword::try_from_interned_id(interner.intern_str(name)),
                    None,
                    "non-keyword {name:?} must not convert to a keyword",
                );
            }
        }
    };
}

keyword_cases! {
    Struct => "struct",
    Enum => "enum",
    Import => "import",
    Export => "export",
    Bind => "bind",
    Alias => "alias",
    Let => "let",
    Change => "change",
    As => "as",
    Var => "var",
    Nest => "nest",
    Complex => "complex",
    Override => "override",
    In => "in",
    For => "for",
    Virtual => "virtual",
}
