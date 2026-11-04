//! Module graph building parser that understands just enough to get `Bind` and `Import`
use algoc::bytes;
use chrn_utils::id_types::SourceId;
use chrn_utils::source_utils::source_manager::SourceManager;
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

const AS_BYTES: &[u8; 2] = b"as";
const IMPORT_BYTES: &[u8; 6] = b"import";
const BIND_BYTES: &[u8; 4] = b"bind";
const VIRTUAL_BYTES: &[u8; 7] = b"virtual";

// Not a good name. This error is general.
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

#[derive(Clone, Copy)]
/// Metadata associated with bytes that are assumed to be a path
struct ReadPathMetadata {
    flags: u8,
}

impl ReadPathMetadata {
    /// Whether or not a backslash is present
    const FLAG_HAS_BACKSLASH: u8 = 1;
    /// Whether or not an end quote is present
    const FLAG_HAS_END_Q: u8 = 2;
    fn new(flags: u8) -> Self {
        Self { flags }
    }
    fn has_end_q(self) -> bool {
        (self.flags & Self::FLAG_HAS_END_Q) != 0
    }
    fn has_backslash(self) -> bool {
        (self.flags & Self::FLAG_HAS_BACKSLASH) != 0
    }
}

/// How to parse a path
enum SourceParseKind {
    // This isn't called virtual because virtual implies in memory, but in memory does not imply virtual
    InMemory,
    Filesystem,
}

/// Mini-parser that uses minimal syntax detection to collect `Import` and `Bind`
pub struct ModuleFinder<'a> {
    /// Current module cursor
    cursor: BasicCursor<'a, u8>,
    cfg: &'a ChrnConfig,
    summary: SourceDiagnosticSummary,
    // Maybe make this general information if anything later wants this info
    // Or loader coult generate this
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
            current_region,
            summary: SourceDiagnosticSummary::default(),
        }
    }

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

            match b {
                b'"' | b'\'' => {
                    self.cursor.skip(1);
                    self.skip_quotes(b);
                }
                b'i' | b'b' => {
                    let prev_ch = self.peek_behind_char();

                    let at_file_start = self.cursor.pos() == 0
                        || (self.cursor.pos() == keywords::EMBEDDING_CLAUSE_SIZE
                            && &self.cursor.items[0..keywords::EMBEDDING_CLAUSE_SIZE] == b"@def");

                    //NOTE: Might tokenize.

                    // Catching "/* */import" so that it doesn't interpret '/import' as an invalid keyword

                    // These checks are needed for the parser to stay simple
                    // The `#` check is needed so that cases like "let x = e#import" do not get
                    // picked up as valid.
                    // `_` is a valid identifier character so need explicit skip
                    let can_check =
                        (!prev_ch.is_alphanumeric() && prev_ch != '#' && prev_ch != '_')
                            || at_file_start;

                    if can_check {
                        //TODO: Both have the exact same flow, but encoding mutation is still a
                        //possible issue. Maybe just share smaller parts?
                        if b == b'i' {
                            let maybe_imp_slice = self.slice_as_kw(IMPORT_BYTES.len());
                            if maybe_imp_slice != IMPORT_BYTES {
                                self.cursor.skip(1);
                                continue;
                            }
                            self.cursor.skip(IMPORT_BYTES.len());

                            // Reject identifier continuations but allow scenarios like
                            // `import"thingy.chrn"`
                            if self.peek_char().is_alphanumeric() || self.peek() == b'_' {
                                continue;
                            }

                            self.skip_comments_and_whitespace();
                            let parse_kind = self.parse_path_modifier();
                            self.skip_comments_and_whitespace();

                            //NOTE: Does nothing for now
                            if module_concepts::STR_PREFIXES.contains(&self.peek()) {
                                self.advance_char();
                            }

                            // May be of concern
                            if self.peek() != b'"' {
                                continue;
                            }

                            let import = match self.parse_import(interner, parse_kind, src_manager)
                            {
                                Ok(i) => i,
                                Err(QuoteError::Diagnostic(d)) => {
                                    self.summary.push_diag(d);
                                    continue;
                                }
                                Err(QuoteError::Unclosed) => break,
                                Err(QuoteError::Ignore) => continue,
                            };

                            imports.push(import);
                        } else if b == b'b' {
                            let maybe_bind_slice = self.slice_as_kw(BIND_BYTES.len());
                            if maybe_bind_slice != BIND_BYTES {
                                self.cursor.skip(1);
                                continue;
                            }
                            self.cursor.skip(BIND_BYTES.len());

                            if self.peek_char().is_alphanumeric() || self.peek() == b'_' {
                                continue;
                            }

                            self.skip_comments_and_whitespace();

                            let parse_kind = self.parse_path_modifier();

                            self.skip_comments_and_whitespace();

                            if module_concepts::STR_PREFIXES.contains(&self.peek()) {
                                self.advance_char();
                            }

                            // May be of concern
                            if self.peek() != b'"' {
                                continue;
                            }

                            bind = match self.parse_bind(src_manager, parse_kind, interner) {
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
                _ => unreachable!(),
            }
        }
        // assert_eq!(imports.len(), 1);

        let mut summary = SourceDiagnosticSummary::default();
        summary.append_summary(&mut self.summary);
        (bind, imports, summary)
    }

    /// Assumes the starting point is at the opening quote
    fn parse_import(
        &mut self,
        interner: &mut Intern,
        parse_kind: SourceParseKind,
        src_manager: &mut SourceManager,
    ) -> Result<Import, QuoteError> {
        // Is `Option` so an alias can cover a non-UTF-8 file name
        let (initial_name_id_opt, src_id, src_span) =
            self.handle_path(interner, parse_kind, src_manager)?;

        // import "main.chrn" as {ident}
        self.skip_comments_and_whitespace();

        let slice = self.slice_as_kw(AS_BYTES.len());

        let alias_id: Option<SpannedContainer<InternedIdUTF8>> = if slice == AS_BYTES {
            self.cursor.skip(AS_BYTES.len());
            self.skip_comments_and_whitespace();

            let start = self.cursor.pos() as u32;
            if self.slice_up_to(crate::ESCAPE_PREFIX.len()) == crate::ESCAPE_PREFIX.as_bytes() {
                self.cursor.skip(crate::ESCAPE_PREFIX.len());
            }

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

        // NOTE: This could be virtual OR a path. May want to route different msgs instead of conernging
        // to generirccc

        // Allows for aliases to be used if import's path is not UTF-8
        let name_id = if let Some(id) = initial_name_id_opt {
            id
        } else if let Some(SpannedContainer { inner, .. }) = alias_id {
            inner
        } else {
            let core_msg = format!(
                "Identifier \"{}\" contains non-UTF-8",
                src_manager.display_source(src_id, interner)
            );

            let src_diag = SourceDiagnostic::builder(
                ErrorCode::ImportErr.into(),
                DiagnosticLevel::Error,
                core_msg,
                self.current_region.src_id,
            )
            .add_annotation(src_span, AnnotationKind::Primary, None)
            //TODO: This is more of a warn than error because the compiler still functions
            // fine without the identifier.
            .add_note("Non-UTF-8 is accepted as a path but requires an `as` alias so the module can still be referenced in-file")
            .build();

            return Err(QuoteError::Diagnostic(src_diag));
        };

        let import_kind = ImportKind::UnresolvedSource(SpannedContainer::new(src_id, src_span));
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

    /// Assumes the starting point is at the opening quote
    fn parse_bind(
        &mut self,
        src_manager: &mut SourceManager,
        parse_kind: SourceParseKind,
        interner: &mut Intern,
    ) -> Result<Bind, QuoteError> {
        // Doesn't need ident since binds are not referenced
        let (_, src_id, src_span) = self.handle_path(interner, parse_kind, src_manager)?;
        let bind = Bind::new(src_id, src_span);

        self.cfg.logger().log_dbg(|| {
            let bind_display = &src_manager.display_source(src_id, interner);
            let region_path = src_manager.display_source(self.current_region.src_id, interner);
            format!(
                "Created bind which points to `{}` for region \"{}\"",
                bind_display, region_path
            )
        });

        Ok(bind)
    }

    /// If there is a path modifier such as `virtual` at the cursor, returns it's corresponding
    /// `SourceParseKind`
    ///
    /// Does no mutation if no modifier is present
    /// Expects that comments and whitespace were already already skipped.
    fn parse_path_modifier(&mut self) -> SourceParseKind {
        // Could just have len set to largest KW then end == VIRTUAL_BYTES.len()
        let maybe_kind = self.slice_as_kw(VIRTUAL_BYTES.len());

        if maybe_kind == VIRTUAL_BYTES {
            self.cursor.skip(VIRTUAL_BYTES.len());
            SourceParseKind::InMemory
        } else {
            SourceParseKind::Filesystem
        }
    }

    /// Wraps `pathbuf_from_bytes` to return a diagnostic
    fn create_pathbuf(&self, slice: &[u8]) -> Result<PathBuf, SourceDiagnostic> {
        match files::pathbuf_from_bytes(slice) {
            Ok(p) => Ok(p),
            Err(kind) => match kind {
                PathConversionErrorKind::InvalidUTF8 => {
                    let msg = "Invalid UTF-8 found within path".to_string();
                    let src_diag = SourceDiagnostic::builder(
                        None,
                        DiagnosticLevel::Error,
                        msg,
                        self.current_region.src_id,
                    )
                    //TODO:
                    // .add_help("Use a p-string p\"...\" to use non-UTF-8 in path")
                    .build();

                    Err(src_diag)
                }
            },
        }
    }

    /// Assumes the starting point is one after the opening quote
    /// Returns (end_cursor, `ReadPathMetadata`)
    ///
    /// Ends one byte after quote end if `FLAG_HAS_END_Q` is `true`
    fn read_path(&mut self) -> (usize, ReadPathMetadata) {
        let mut flags = 0;
        while self.cursor.pos() < self.cursor.len() {
            match self.peek() {
                b'"' => {
                    self.cursor.skip(1);
                    flags |= ReadPathMetadata::FLAG_HAS_END_Q;
                    break;
                }
                b'\\' => {
                    flags |= ReadPathMetadata::FLAG_HAS_BACKSLASH;
                    self.cursor.skip(1);
                }
                _ => {
                    self.cursor.skip(1);
                }
            }
        }

        let end_cursor = self.cursor.pos();
        (end_cursor, ReadPathMetadata::new(flags))
    }

    /// Utility for `Import` and `Bind` to share handling of their source processing
    /// Expects that comments and whitespace were already already skipped.
    fn handle_path(
        &mut self,
        interner: &mut Intern,
        parse_kind: SourceParseKind,
        src_manager: &mut SourceManager,
        // This is only `Option` because files can exchange their module ident with their alias, but
        // something like virtual MUST have a valid identifier.
    ) -> Result<(Option<InternedIdUTF8>, SourceId, SourceSpan), QuoteError> {
        let start_cursor = self.cursor.pos();
        self.cursor.skip(1);

        let (end_cursor, read_path_meta) = self.read_path();

        if !read_path_meta.has_end_q() {
            return Err(QuoteError::Unclosed);
        }

        let path_span = SourceSpan::new(
            self.current_region.self_id,
            // Includes start quote
            start_cursor as u32,
            // Includes end quote
            end_cursor as u32,
        );

        // Solely for the intent of giving a more descriptive error
        if read_path_meta.has_backslash() {
            let core_msg = "Only '/' can be used as path separators.".to_string();
            let builder = SourceDiagnostic::builder(
                ErrorCode::ImportErr.into(),
                DiagnosticLevel::Error,
                core_msg,
                self.current_region.src_id,
            )
            .add_annotation(path_span, AnnotationKind::Primary, None);
            return Err(QuoteError::Diagnostic(builder.build()));
        }

        // To exclude start quote
        // To exclude end quote
        let slice = &self.cursor.items[start_cursor + 1..end_cursor - 1];

        self.skip_comments_and_whitespace();

        let (name_id_opt, src_id) = match parse_kind {
            SourceParseKind::InMemory => {
                let ident = match str::from_utf8(slice) {
                    Ok(s) => s,
                    Err(_) => {
                        // Will route to different strings if ever needed
                        let core_msg = "Virtual source must be valid UTF-8";
                        //TODO: ErrorCode
                        let builder = SourceDiagnostic::builder(
                            None,
                            DiagnosticLevel::Error,
                            core_msg,
                            self.current_region.src_id,
                        )
                        .add_annotation(path_span, AnnotationKind::Primary, None)
                        .add_note("Unlike file paths, in-memory sources must be valid UTF-8");
                        return Err(QuoteError::Diagnostic(builder.build()));
                    }
                };
                let interned_id = match interner.get_interned_id_from_str(&ident) {
                    Some(id) => id,
                    None => {
                        let core_msg = format!("Could not find module `{ident}` in memory");
                        //TODO: ErrorCode
                        let builder = SourceDiagnostic::builder(
                            None,
                            DiagnosticLevel::Error,
                            core_msg,
                            self.current_region.src_id,
                        )
                        .add_annotation(
                            path_span,
                            AnnotationKind::Primary,
                            None,
                        );
                        return Err(QuoteError::Diagnostic(builder.build()));
                    }
                };
                let src_id = match src_manager.get_source_id_from_interned_id(interned_id) {
                    Some(id) => id,
                    None => {
                        let core_msg = format!("Could not find module `{ident}` in memory");
                        //TODO: ErrorCode
                        let builder = SourceDiagnostic::builder(
                            None,
                            DiagnosticLevel::Error,
                            core_msg,
                            self.current_region.src_id,
                        )
                        .add_annotation(
                            path_span,
                            AnnotationKind::Primary,
                            None,
                        );
                        return Err(QuoteError::Diagnostic(builder.build()));
                    }
                };
                (Some(interned_id), src_id)
            }
            SourceParseKind::Filesystem => {
                let path_buf = self.create_pathbuf(slice)?;
                let import_path = match path_buf.canonicalize() {
                    Ok(p) => p,
                    Err(e) => {
                        let core_msg = core_error::form_string_from_io_err(&e, &path_buf)
                            .unwrap_or(e.to_string());
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
                // If an alias is given the file is a valid source
                let name_id_opt = match import_path.file_prefix().map(|n| n.to_str()).flatten() {
                    Some(s) => Some(interner.intern_str(s)),
                    None => None,
                };
                let src_id = src_manager.register_path(&import_path).inner();
                (name_id_opt, src_id)
            }
        };
        Ok((name_id_opt, src_id, path_span))
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
            && (self.peek_char().is_alphanumeric() || self.peek() == b'_')
        {
            self.advance_char();
        }

        let end = self.cursor.pos();

        // Enforces utf-8 but module paths themselves don't need to be valid utf-8, am I
        // hallucinating?
        // Yes, yes.
        let id_str = str::from_utf8(&self.cursor.items[start..end])
            .expect("Loop should only accept valid UTF-8 characters.");

        if id_str.len() == 0 {
            return None;
        }

        Some(interner.intern_str(&id_str))
    }

    /// Returns previous character
    fn peek_behind_char(&self) -> char {
        let prefix = &self.cursor.items[..self.cursor.pos()];
        // NOTE: There are many different ways to handle this but keeping this route for now.
        // Checking at most if 4 bytes are occupied for UTF-8
        let Some(offset) = prefix
            .iter()
            .rev()
            .take(4)
            .position(|byte| byte & 0xC0 != 0x80)
        else {
            return '\0';
        };
        let chunk = &prefix[prefix.len() - offset - 1..];

        std::str::from_utf8(chunk)
            .ok()
            .and_then(|c| c.chars().next())
            .unwrap_or('\0')
    }

    /// Expects to start 1 after the start quote.
    fn skip_quotes(&mut self, q_type: u8) {
        debug_assert!(q_type == b'\'' || q_type == b'"');
        while self.cursor.pos() < self.cursor.len() {
            match self.peek() {
                // Bare minimum is skipping \", otherwise this doesn't really matter
                b'\\' => self.cursor.skip(2),
                b if b == q_type => {
                    self.cursor.skip(1);
                    break;
                }
                _ => {
                    self.cursor.skip(1);
                }
            }
        }
    }

    //TODO: Should maybe be a general function
    /// Slices using keyword rules
    ///
    /// Returns `false` if the slice would exceed the length or `bytes` was not matched
    fn slice_as_kw(&self, kw_len: usize) -> &[u8] {
        let start = self.cursor.pos();
        let mut end = self.cursor.pos();
        let len = start + (kw_len);

        while end < self.cursor.len() && self.cursor.items[end].is_ascii_alphabetic() && end < len {
            end += 1;
        }

        &self.cursor.items[start..end]
    }

    //TODO: Inside `BasicCursor`?
    fn slice_up_to(&self, len: usize) -> &[u8] {
        let start = self.cursor.pos();
        let mut end = self.cursor.pos();
        let len = start + (len);

        while end < self.cursor.len() && end < len {
            end += 1;
        }

        &self.cursor.items[start..end]
    }

    fn skip_single_comment(&mut self) {
        while self.cursor.pos() < self.cursor.len() && self.peek() != b'\n' {
            self.cursor.skip(1);
        }
    }

    fn skip_multi_comment(&mut self) {
        let mut depth = 1;

        while self.cursor.pos() < self.cursor.len() && depth > 0 {
            if self.peek() == b'/' && self.peek_ahead(1) == b'*' {
                self.cursor.skip(2);
                depth += 1;
            } else if self.peek() == b'*' && self.peek_ahead(1) == b'/' {
                self.cursor.skip(2);
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
            if self.cursor.pos() >= self.cursor.len()
                || b == b'i'
                || b == b'b'
                || b == b'"'
                || b == b'\''
            {
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
