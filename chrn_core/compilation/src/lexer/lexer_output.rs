use crate::lexer::{token::SpannedToken, trivia::Trivia};

/// Container for lexer-specific output data
pub struct LexerOutput {
    // Not even going to comment on what was here before.
    /// Tokens collected by Lexer
    pub toks: Vec<SpannedToken>,
    /// Trivia collected by Lexer
    pub trivia: Vec<Trivia>,
    /// Every index of an `Token::HashSymbol` instance
    ///
    /// Collected so that directives can be processed without walking all of `self.toks`
    pub hash_tok_indices: Vec<usize>,
    /// Amount of `Token::Invalid` spotted
    pub found_invalid_toks: u8,
}

impl LexerOutput {
    pub const fn new(
        toks: Vec<SpannedToken>,
        trivia: Vec<Trivia>,
        hash_tok_indices: Vec<usize>,
        found_invalid_toks: u8,
    ) -> Self {
        Self {
            toks,
            trivia,
            hash_tok_indices,
            found_invalid_toks,
        }
    }
}
