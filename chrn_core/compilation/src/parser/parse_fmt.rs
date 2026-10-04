use chrn_utils::intern::Intern;
use lang::chrn_classifier::ChrnClassifiable;

use crate::lexer::token::{Token, TokenFloat, TokenInt};

/// Helper to reduce boiler-plate of formatting a given token
pub(super) fn fmt_tok(tok: Token, interner: &Intern) -> String {
    let fmtted = match tok {
        Token::Def => "`@def`".to_string(),
        Token::End => "`@end`".to_string(),
        Token::Id(name_id)
        | Token::Str(name_id)
        | Token::Integer(TokenInt {
            interned_id: name_id,
            ..
        })
        | Token::Float(TokenFloat {
            interned_id: name_id,
            ..
        }) => {
            let ident = interner.search(name_id);
            format!("\"{ident}\"")
        }
        Token::Keyword(kw) => format!("keyword `{}`", kw.to_classified().to_string()),
        Token::Invalid(name_id) => {
            let invalid_msg = interner.search(name_id);
            let new_msg = format!("invalid token \"{invalid_msg}\"");
            new_msg
        }
        Token::Char(ch) => format!("'{ch}'"),
        Token::BoolLiteral(boolean) => format!("bool literal `{}`", boolean),
        t => format!("`{}`", t.kind()),
    };

    fmtted
}
