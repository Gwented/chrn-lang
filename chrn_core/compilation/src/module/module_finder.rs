//! Module graph building parser that understands just enough to get `Bind` and `Import`
use algoc::bytes;
use chrn_utils::source_utils::source_manager::{SourceInMemory, SourceKind, SourceManager};
use std::path::PathBuf;

use chrn_utils::err_codes::ErrorCode;
use chrn_utils::platform::files;
use chrn_utils::platform::files::error::PathConversionErrorKind;
use chrn_utils::source_utils::source_diagnostic::annotations::AnnotationKind;
use chrn_utils::source_utils::source_diagnostic::{
    DiagnosticLevel, SourceDiagnosticSink, SourceDiagnosticSummary,
};
use chrn_utils::utils::containers::SpannedContainer;
use chrn_utils::{
    core_error::{self},
    id_types::InternedIdUTF8,
    intern::Intern,
    source_utils::{
        source_diagnostic::SourceDiagnostic, source_region::SourceRegion, source_span::SourceSpan,
    },
};
use lang::keywords;
use toolsc::cursors::BasicCursor;

use crate::chrn_config::ChrnConfig;
use crate::module::module_concepts::{self, Bind};
use crate::module::{Import, ImportKind};

/// Mini-parser that uses minimal syntax detection to collect `Import` and `Bind`
pub struct ModuleFinder<'a> {
    /// Current module cursor
    cursor: BasicCursor<'a, u8>,
    cfg: &'a ChrnConfig,
    summary: SourceDiagnosticSummary,
    // Maybe make this general information if anything later wants this info
    // Or loader coult generate this
    /// Prevents `peek_behind_char` from potentially reading p"" strings which do not need to be UTF-8
    last_end_quote_pos: Option<usize>,
    //// Allows for excluding comments from checks
    // last_comment_end: Option<usize>,
    /// Path origin so that errors can accurately report the path where the import was declared
    current_region: &'a SourceRegion,
}

impl ModuleFinder<'_> {
    pub fn new<'a>(
        // WARN: Could make it clearer that these source bytes SHOULD be curated where there are no
        // unclosed quotes or comments since chrn does not create regions for those.
        src_bytes: &'a [u8],
        cfg: &'a ChrnConfig,
        current_region: &'a SourceRegion,
    ) -> ModuleFinder<'a> {
        ModuleFinder {
            cursor: BasicCursor::new(src_bytes),
            cfg,
            last_end_quote_pos: None,
            // last_comment_end: None,
            current_region,
            summary: SourceDiagnosticSummary::default(),
        }
    }

    //TODO: Refine this system. Should probably not prefer methods like "is_as()" actually mutating
    //and instead do manual assignment, or at least make 2 separate versions where it does the assignment.
    //Or not idk.

    // Collect imports isn't the right name, but collect (em-dash) what?
    //FIXME: Several several bugs
    /// Collects `Import` and `Bind` where possible. Reports upon stage-specific errors related to
    /// import/bind. Ignores others such as unclosed syntax.
    pub fn collect_imports(
        &mut self,
        interner: &mut Intern,
        src_manager: &mut SourceManager,
    ) -> (Option<Bind>, Vec<Import>, SourceDiagnosticSummary) {
        let mut imports: Vec<Import> = Vec::new();
        let mut bind: Option<Bind> = None;

        loop {
            self.skip_until_important();

            if self.cursor.pos() >= self.cursor.len() {
                break;
            }

            let b = self.peek();

            //TODO: Maybe redo all
            match b {
                b'"' => {
                    self.cursor.skip(1);
                    self.skip_quotes()
                }
                c if c == b'i' || c == b'b' => {
                    let prev_ch = self.peek_behind_char();

                    let at_file_start = self.cursor.pos() == 0
                        || (self.cursor.pos() == keywords::EMBEDDING_CLAUSE_SIZE
                            && &self.cursor.items[0..keywords::EMBEDDING_CLAUSE_SIZE] == b"@def");

                    // Catching "/* */import" so that it doesn't interpret '/' as an invalid keyword

                    // These checks are needed for the parser to stay simple
                    let can_check = !prev_ch.is_alphanumeric() || at_file_start;

                    if can_check {
                        if c == b'i' && self.is_import() {
                            let import = match self.parse_import(interner, src_manager) {
                                Ok(i) => i,
                                Err(QuoteError::Diagnostic(d)) => {
                                    self.summary.push_diag(d);
                                    continue;
                                }
                                Err(QuoteError::Unclosed) => break,
                                Err(QuoteError::Ignore) => continue,
                            };

                            imports.push(import);
                        } else if c == b'b' && self.is_bind() {
                            bind = match self.parse_bind(src_manager, interner) {
                                Ok(b) => Some(b),
                                Err(QuoteError::Diagnostic(d)) => {
                                    self.summary.push_diag(d);
                                    continue;
                                }
                                Err(QuoteError::Unclosed) => break,
                                Err(QuoteError::Ignore) => continue,
                            };
                        }
                    } else {
                        // skipping i/b
                        self.cursor.skip(1);
                    }
                }
                _ => {
                    self.cursor.skip(1);
                }
            }
        }

        let mut summary = SourceDiagnosticSummary::default();
        summary.append_summary(&mut self.summary);
        (bind, imports, summary)
    }

    /// Assumes the starting point is at the opening quote
    fn parse_import(
        &mut self,
        interner: &mut Intern,
        src_manager: &mut SourceManager,
    ) -> Result<Import, QuoteError> {
        self.cursor.skip(1);
        let start_cursor = self.cursor.pos();
        // Boolean to track if a "\" was seen since only "/" can be used to separate
        let mut saw_backslash = false;
        let mut has_end_q = false;

        self.skip_comments_and_whitespace();
        while self.cursor.pos() < self.cursor.len() {
            match self.peek() {
                b'"' => {
                    self.last_end_quote_pos = Some(self.cursor.pos());
                    self.cursor.skip(1);
                    has_end_q = true;
                    break;
                }
                b'\\' => {
                    saw_backslash = true;
                    self.cursor.skip(1);
                }
                _ => {
                    self.cursor.skip(1);
                }
            }
        }

        if !has_end_q {
            return Err(QuoteError::Unclosed);
        }

        // - 1 for same reason as in lexer. Before breaking the last quote is skipped, and since
        // span ends are exclusive, the end pos would be one after the end quote, so we need to go
        // back 1 to properly sit at the end quote
        let end_cursor = self.cursor.pos() - 1;

        let path_span = SourceSpan::new(
            self.current_region.self_id,
            // To include start quote
            start_cursor as u32,
            // To include end quote
            end_cursor as u32,
        );

        // Solely for the intent of giving a more descriptive error
        if saw_backslash {
            let core_msg = "Only '/' can be used as path separators.".to_string();
            //NOTE: Maybe an error code?
            let src_diag = SourceDiagnostic::builder(
                ErrorCode::ImportErr.into(),
                DiagnosticLevel::Error,
                core_msg,
                self.current_region.src_id,
            )
            .add_annotation(path_span, AnnotationKind::Primary, None)
            .build();

            return Err(QuoteError::Diagnostic(src_diag));
        }

        //TODO: Need to have a path for in memory
        let path_buf = self.create_pathbuf(&self.cursor.items[start_cursor..end_cursor])?;

        let import_path = match path_buf.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                let core_msg =
                    core_error::form_string_from_io_err(&e, &path_buf).unwrap_or(e.to_string());
                let src_diag = SourceDiagnostic::builder(
                    ErrorCode::ImportErr.into(),
                    DiagnosticLevel::Error,
                    core_msg,
                    self.current_region.src_id,
                )
                .add_annotation(path_span, AnnotationKind::Primary, None)
                .build();

                return Err(QuoteError::Diagnostic(src_diag));
            }
        };

        // We could check for an alias here too in case the file name is invalid and needs an alias
        //TODO:
        let file_name_opt = import_path.file_prefix().map(|n| n.to_str()).flatten();

        let path_id = src_manager.register_path(&import_path);

        // import "main.chrn" as {ident}
        self.skip_comments_and_whitespace();
        let alias_id: Option<SpannedContainer<InternedIdUTF8>> = if self.is_as() {
            self.skip_comments_and_whitespace();
            let start = self.cursor.pos() as u32;
            let Some(id) = self.read_id(interner) else {
                return Err(QuoteError::Ignore);
            };
            SpannedContainer::new(
                id,
                SourceSpan::new(self.current_region.self_id, start, self.cursor.pos() as u32),
            )
            .into()
        } else {
            None
        };

        // Allows for aliases to be used if import's path is not UTF-8
        let name_id = if let Some(name) = file_name_opt {
            interner.intern_str(name)
        } else if let Some(SpannedContainer { inner, .. }) = alias_id {
            inner
        } else {
            let core_msg = format!("Path \"{}\" contains non-UTF-8", import_path.display());

            //TODO: Aliasing
            let src_diag = SourceDiagnostic::builder(
                ErrorCode::ImportErr.into(),
                DiagnosticLevel::Error,
                core_msg,
                self.current_region.src_id,
            )
            .add_annotation(path_span, AnnotationKind::Primary, None)
            //TODO: This is more of a warn than error because the compiler still functions
            // fine without the identifier.
            .add_note("Non-UTF-8 is accepted as a file path but requires an `as` alias so the module can still be referenced in-file")
            .build();

            return Err(QuoteError::Diagnostic(src_diag));
        };

        let import_kind = ImportKind::UnresolvedSource(SpannedContainer::new(path_id, path_span));
        let import = Import::new(name_id, import_kind, alias_id);

        self.cfg.logger().log_dbg(|| {
            let import_name = interner.extract_utf8_str(name_id);
            let region_path = src_manager.display_source(self.current_region.src_id, interner);
            format!(
                "Created import `{import_name}` (region=\"{}\")",
                region_path
            )
        });

        Ok(import)
    }

    //WARN: Will be placed in different module
    fn create_pathbuf(&self, slice: &[u8]) -> Result<PathBuf, SourceDiagnostic> {
        match files::pathbuf_from_bytes(slice) {
            Ok(p) => Ok(p),
            Err(kind) => match kind {
                PathConversionErrorKind::InvalidUTF8 => {
                    let msg = "Invalid UTF-8 found within file".to_string();
                    let src_diag = SourceDiagnostic::builder(
                        None,
                        DiagnosticLevel::Error,
                        msg,
                        self.current_region.src_id,
                    )
                    .build();

                    Err(src_diag)
                }
            },
        }
    }

    /// Assumes the starting point is at the opening quote
    fn parse_bind(
        &mut self,
        src_manager: &mut SourceManager,
        interner: &Intern,
    ) -> Result<Bind, QuoteError> {
        // skipping "
        self.cursor.skip(1);
        let start_cursor = self.cursor.pos();

        let mut saw_backslash = false;
        let mut has_end_q = false;

        self.skip_comments_and_whitespace();
        while self.cursor.pos() < self.cursor.len() {
            match self.peek() {
                b'"' => {
                    self.cursor.skip(1);
                    has_end_q = true;
                    break;
                }
                b'\\' => {
                    saw_backslash = true;
                    self.cursor.skip(1);
                }
                _ => {
                    self.cursor.skip(1);
                }
            }
        }

        if !has_end_q {
            return Err(QuoteError::Unclosed);
        }

        let end_cursor = self.cursor.pos() - 1;

        let path_span = SourceSpan::new(
            self.current_region.self_id,
            start_cursor as u32,
            end_cursor as u32,
        );

        // Solely for the intent of giving a more descriptive error
        if saw_backslash {
            let core_msg = "Only '/' can be used as path separators.".to_string();

            let src_diag = SourceDiagnostic::builder(
                None,
                DiagnosticLevel::Error,
                core_msg,
                self.current_region.src_id,
            )
            .add_annotation(path_span, AnnotationKind::Primary, None)
            .build();

            return Err(QuoteError::Diagnostic(src_diag));
        }

        // WHY WAS THIS UNWRAP FOR SO LONG
        let path_buf = self.create_pathbuf(&self.cursor.items[start_cursor..end_cursor])?;

        let bind_path = match path_buf.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                let core_msg =
                    core_error::form_string_from_io_err(&e, &path_buf).unwrap_or(e.to_string());

                let src_diag = SourceDiagnostic::builder(
                    None,
                    DiagnosticLevel::Error,
                    core_msg,
                    self.current_region.src_id,
                )
                .add_annotation(path_span, AnnotationKind::Primary, None)
                .build();

                return Err(QuoteError::Diagnostic(src_diag));
            }
        };

        let bind_path_id = src_manager.register_path(&bind_path);
        let bind = Bind::new(bind_path_id, path_span);

        self.cfg.logger().log_dbg(|| {
            let bind_path = &src_manager.extract_path(bind_path_id).path;
            let region_path = src_manager.display_source(self.current_region.src_id, interner);
            format!(
                "Created bind which points to `{}` for region \"{}\"",
                bind_path.display(),
                region_path
            )
        });

        Ok(bind)
    }

    /// Starting from cursor reads identifier
    fn read_id(&mut self, interner: &mut Intern) -> Option<InternedIdUTF8> {
        let start = self.cursor.pos();

        if self.peek().is_ascii_digit() {
            return None;
        }

        //TODO: Maybe encode this where alphanumeric || '_' being accepted iff the first character
        //is not alphanmueric is a valid identifier.
        while self.cursor.pos() < self.cursor.len()
            && (self.peek_char().is_alphanumeric()
                || self.cursor.pos() < self.cursor.len() && self.peek() == b'_')
        {
            self.advance_char();
        }

        let end = self.cursor.pos();

        // Enforces utf-8 but module paths themselves don't need to be valid utf-8, am I
        // hallucinating?
        // Yes, yes.
        let id_str = str::from_utf8(&self.cursor.items[start..end])
            .expect("Cannot fail due to loop only accepting valid UTF-8 characters.");

        Some(interner.intern_str(&id_str))
    }

    /// Returns previous character
    fn peek_behind_char(&self) -> char {
        let start = self.last_end_quote_pos.unwrap_or_default();
        // Inclusive since otherwise it would skip the current character and there would need to be
        // a saturating sub to make up for it
        let chunk = &self.cursor.items[start..=self.cursor.pos()];

        std::str::from_utf8(chunk)
            .ok()
            .and_then(|c| c.chars().rev().skip(1).next())
            .unwrap_or('\0')
    }

    /// Expects to start 1 after the start quote.
    fn skip_quotes(&mut self) {
        while self.cursor.pos() < self.cursor.len() {
            match self.peek() {
                // Bare minimum is skipping \", otherwise this doesn't really matter
                b'\\' => self.cursor.skip(2),
                b'"' => {
                    self.last_end_quote_pos = Some(self.cursor.pos());
                    self.cursor.skip(1);
                    break;
                }
                _ => {
                    self.cursor.skip(1);
                }
            }
        }
    }

    fn read_escape(&mut self) -> Option<char> {
        match self.peek() {
            b'n' => {
                self.cursor.skip(1);
                Some('\n')
            }
            b'r' => {
                self.cursor.skip(1);
                Some('\r')
            }
            b't' => {
                self.cursor.skip(1);
                Some('\t')
            }
            b'\\' => {
                self.cursor.skip(1);
                Some('\\')
            }
            b'0' => {
                self.cursor.skip(1);
                Some('\0')
            }
            b'\'' => {
                self.cursor.skip(1);
                Some('\'')
            }
            b'"' => {
                self.cursor.skip(1);
                Some('"')
            }
            b'x' => {
                self.cursor.skip(1);
                let mut val: u8 = 0;
                let mut count = 0;

                while count < 2 {
                    let c = self.peek();

                    let digit = match c {
                        b'0'..=b'9' => c - b'0',
                        b'a'..=b'f' => c - b'a' + 10,
                        b'A'..=b'F' => c - b'A' + 10,
                        _ => break,
                    };

                    val = (val << 4) | digit;
                    self.cursor.skip(1);
                    count += 1;
                }

                if count == 2 {
                    let next = self.peek();
                    if matches!(next, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') {
                        None
                    } else {
                        Some(val as char)
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn is_import(&mut self) -> bool {
        let start = self.cursor.pos();

        while self.cursor.pos() < self.cursor.len() && self.peek().is_ascii_alphabetic() {
            self.cursor.skip(1);
        }

        let end = self.cursor.pos();

        if &self.cursor.items[start..end] != b"import" {
            return false;
        }

        self.skip_comments_and_whitespace();

        //WARN: If ever exceeds just `u8` will need different check
        let prev_byte = self.cursor.peek_behind_owned(1);
        if module_concepts::STR_PREFIXES.contains(&prev_byte) {
            self.advance_char();
        }

        if self.peek() != b'"' {
            return false;
        }

        true
    }

    // WARN: Suspicipus
    fn is_as(&mut self) -> bool {
        if self.cursor.pos() + 2 < self.cursor.len()
            && &self.cursor.items[self.cursor.pos()..self.cursor.pos() + 2] == b"as"
        {
            self.cursor.skip(2);
            return true;
        }

        false
    }

    // Maybe route import and bind to same method
    // Should you be able to "promise" binds? No.
    fn is_bind(&mut self) -> bool {
        let start = self.cursor.pos();

        while self.cursor.pos() < self.cursor.len() && self.peek().is_ascii_alphabetic() {
            self.cursor.skip(1);
        }

        let end = self.cursor.pos();

        if &self.cursor.items[start..end] != b"bind" {
            return false;
        }

        self.skip_comments_and_whitespace();

        if module_concepts::STR_PREFIXES.contains(&self.peek()) {
            self.advance_char();
        }

        if self.peek() != b'"' {
            return false;
        }

        true
    }

    fn skip_single_comment(&mut self) {
        while self.cursor.pos() < self.cursor.len() && self.peek() != b'\n' {
            self.cursor.skip(1);
        }
        // self.last_comment_end = Some(self.cursor.pos() - 1);
    }

    fn skip_multi_comment(&mut self) {
        let mut depth = 1;

        while self.cursor.pos() < self.cursor.len() && depth > 0 {
            if self.peek() == b'/' && self.peek_ahead(1) == b'*' {
                self.cursor.skip(2);
                depth += 1;
            } else if self.peek() == b'*' && self.peek_ahead(1) == b'/' {
                self.cursor.skip(2);
                // Ending at '/'
                // self.last_comment_end = Some(self.cursor.pos() - 1);
                depth -= 1;
            } else {
                self.cursor.skip(1);
            }
        }
    }

    fn peek(&self) -> u8 {
        self.cursor.peek_owned_checked().unwrap_or(b'\0')
    }

    fn peek_char(&self) -> char {
        let b = self.peek();

        if b <= 127 {
            return b as char;
        }

        let closest_end = bytes::get_next_ascii_character(self.cursor.items, self.cursor.pos());
        let chunk = &self.cursor.items[self.cursor.pos()..closest_end];

        // Lazy evaluation to avoid utf-8 checking entire self.bytes
        std::str::from_utf8(chunk)
            .ok()
            .and_then(|c| c.chars().next())
            .unwrap_or('\0')
    }

    /// Does not skip quotes since quotes may contain needed information depending on context.
    fn skip_comments_and_whitespace(&mut self) {
        loop {
            self.skip_whitespace();

            let b = self.peek();

            if self.cursor.pos() >= self.cursor.len() {
                break;
            }

            match b {
                b'/' => {
                    if self.peek_ahead(1) == b'/' {
                        self.cursor.skip(2);
                        self.skip_single_comment();
                    } else if self.peek_ahead(1) == b'*' {
                        self.cursor.skip(2);
                        self.skip_multi_comment();
                    } else {
                        self.cursor.skip(1);
                    }
                }
                _ => break,
            }
        }
    }

    fn advance_char(&mut self) -> char {
        let ch = self.peek_char();
        self.cursor.skip(ch.len_utf8());
        ch
    }

    fn peek_ahead(&self, amt: usize) -> u8 {
        self.cursor.peek_ahead_owned_checked(amt).unwrap_or(b'\0')
    }

    fn skip_until_important(&mut self) {
        loop {
            // Should there be skip_comments_whitespace_quotes?
            self.skip_comments_and_whitespace();
            let b = self.peek();
            if self.cursor.pos() >= self.cursor.len() || b == b'i' || b == b'b' || b == b'"' {
                break;
            }
            self.cursor.skip(1);
        }
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_ascii_whitespace() {
            self.cursor.skip(1);
        }

        while self.peek_char().is_whitespace() {
            self.advance_char();
        }
    }
}

enum QuoteError {
    Diagnostic(SourceDiagnostic),
    Unclosed,
    Ignore,
}

impl From<SourceDiagnostic> for QuoteError {
    fn from(d: SourceDiagnostic) -> Self {
        QuoteError::Diagnostic(d)
    }
}
