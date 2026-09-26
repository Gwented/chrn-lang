use crate::lexer::token::TokenKind;

/// Encodes a named start, end, and arg separator
#[derive(Debug, Clone, Copy)]
pub struct DelimiterContext {
    opening: TokenKind,
    arg_sep: TokenKind,
    closing: TokenKind,
}

impl DelimiterContext {
    pub fn new(opening: TokenKind, arg_sep: TokenKind, closing: TokenKind) -> Self {
        Self {
            opening,
            arg_sep,
            closing,
        }
    }

    /// Creates `Self` with `arg_sep` set to `TokenKind::Comma`
    pub fn with_comma(opening: TokenKind, closing: TokenKind) -> Self {
        Self {
            opening,
            arg_sep: TokenKind::Comma,
            closing,
        }
    }

    pub fn opening(&self) -> TokenKind {
        self.opening
    }

    pub fn arg_sep(&self) -> TokenKind {
        self.arg_sep
    }

    pub fn closing(&self) -> TokenKind {
        self.closing
    }
}
