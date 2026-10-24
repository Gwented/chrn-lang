use std::cmp;

use algoc::{bytes, strings};
use lang::{
    chrn_classifier::{ChrnClassifiable, ChrnClassified},
    directive_consts, keywords,
    types::builtins,
};

/// Fuzzy kiwis
#[derive(Clone, Copy)]
pub enum FuzzyMatch {
    KW,
    Type,
    Sect,
    Stmt,
    Directive,
}

impl ChrnClassifiable for FuzzyMatch {
    fn to_classified(&self) -> lang::chrn_classifier::ChrnClassified {
        match self {
            FuzzyMatch::KW => ChrnClassified::KW,
            FuzzyMatch::Type => ChrnClassified::Type,
            FuzzyMatch::Sect => ChrnClassified::AbstractSection,
            FuzzyMatch::Stmt => ChrnClassified::Stmt,
            FuzzyMatch::Directive => ChrnClassified::Directive,
        }
    }
}

// This is a lie.
/// Executes same search as the function `fuzzy_match`
///
/// Returns an `Option` in comparison to `fuzzy_match` because the target's string `Formatted`
/// version may or may not be relevant to return
pub fn fuzzy_match_with_fmtted(
    given: &str,
    target: FuzzyMatch,
) -> Option<(Vec<&str>, ChrnClassified)> {
    Some((fuzzy_match(given, target), target.to_classified()))
}

/// Fuzzily searches stuff
pub fn fuzzy_match(given: &str, target: FuzzyMatch) -> Vec<&str> {
    match target {
        FuzzyMatch::KW => _fuzzy_match(given, &keywords::KEYWORDS_ARRAY),
        FuzzyMatch::Type => _fuzzy_match(given, &builtins::BUILTIN_TYPE_ARRAY),
        FuzzyMatch::Stmt => _fuzzy_match(given, &keywords::KEYWORDS_ARRAY[keywords::stmt_range()]),
        FuzzyMatch::Sect => _fuzzy_match(given, &keywords::KEYWORDS_ARRAY[keywords::sect_range()]),
        FuzzyMatch::Directive => _fuzzy_match(given, &directive_consts::BUILTIN_DIRECTIVE_STRS),
    }
}

/// `given` represents the given bytes that are to be compared to the elements of `arr`.
// Returns option string instead of index because not all arrays are loaded at startup
fn _fuzzy_match<'a>(given: &str, arr: &'a [&str]) -> Vec<&'a str> {
    let mut found = Vec::new();
    if given.len() == 1 {
        return found;
    }

    // Calculating this inline instead of constants due to it being prone to bugs
    let mut max_len = 0;
    for s in arr {
        if s.len() > max_len {
            max_len = s.len();
        }
    }

    // Compensates by exceeding bounds by 3
    if given.len() > max_len + 3 {
        return found;
    }

    for other in arr {
        if strings::is_similar(given, other) {
            found.push(other);
        }
    }

    found
}
