use crate::lexer::token::TokenKind;

/// Encodes a named start, end, and arg separator
#[derive(Debug, Clone, Copy)]
pub(super) struct DelimiterContext {
    opening: TokenKind,
    arg_sep: TokenKind,
    closing: TokenKind,
}

impl DelimiterContext {
    pub(super) fn new(opening: TokenKind, arg_sep: TokenKind, closing: TokenKind) -> Self {
        Self {
            opening,
            arg_sep,
            closing,
        }
    }

    /// Creates `Self` with `arg_sep` set to `TokenKind::Comma`
    pub(super) fn with_comma(opening: TokenKind, closing: TokenKind) -> Self {
        Self {
            opening,
            arg_sep: TokenKind::Comma,
            closing,
        }
    }

    pub(super) fn opening(&self) -> TokenKind {
        self.opening
    }

    pub(super) fn arg_sep(&self) -> TokenKind {
        self.arg_sep
    }

    pub(super) fn closing(&self) -> TokenKind {
        self.closing
    }
}
