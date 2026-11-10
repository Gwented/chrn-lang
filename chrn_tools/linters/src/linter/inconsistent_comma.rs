use chrn_utils::source_utils::source_diagnostic::SourceDiagnostic;
use compilation::{lexer::token::TokenInfo, parser::ast::ast_concepts::AstInfo};

use crate::linter::Lintable;

#[derive(Debug)]
pub struct InconsistentCommaLinter<'a> {
    ast_info: &'a AstInfo,
    toks: &'a [TokenInfo],
}

impl InconsistentCommaLinter<'_> {
    pub fn new<'a>(ast_info: &'a AstInfo, toks: &'a [TokenInfo]) -> InconsistentCommaLinter<'a> {
        InconsistentCommaLinter { ast_info, toks }
    }
}

impl Lintable for InconsistentCommaLinter<'_> {
    fn lint(&self) -> Vec<SourceDiagnostic> {
        for thing in self.ast_info.sections() {
            todo!();
        }

        todo!()
    }
}
