//! Module graph building parser that understands just enough to get `Bind` and `Import`
use std::{ffi::OsStr, path::PathBuf, str::FromStr};

use chrn_utils::err_codes::ErrorCode;
use chrn_utils::source_map::source_diagnostic::annotations::AnnotationKind;
use chrn_utils::source_map::source_diagnostic::{
    DiagnosticLevel, SourceDiagnosticSink, SourceDiagnosticSummary,
};
use chrn_utils::utils::containers::SpannedContainer;
use chrn_utils::{
    core_error::{self},
    id_types::InternedId,
    intern::Intern,
    source_map::{
        source_diagnostic::SourceDiagnostic, source_region::SourceRegion, source_span::SourceSpan,
    },
};
use lang::keywords;
use toolsc::cursors::BasicCursor;

use crate::chrn_config::ChrnConfig;
use crate::module::module_concepts::Bind;
use crate::module::{Import, ImportKind};

/// Module graph start-up structure that finds imports and assigns them a `ModuleId` from `seen`.
///
/// This is not recursive in any way, it's just a mini parser which uses the most minimal syntax
/// possible to search `src_bytes` and identifiy imports and bind usage where possible.
pub struct ModuleFinder<'a> {
    /// Current module cursor
    cursor: BasicCursor<'a, u8>,
    cfg: &'a ChrnConfig,
    summary: SourceDiagnosticSummary,
    /// Path origin so that errors can accurately report the path where the import was declared
    current_region: &'a SourceRegion,
}

impl ModuleFinder<'_> {
    pub fn new<'a>(
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

    /// Returns a tuple with `Bind` and all imports found on `Ok`.
    /// Returns `ConfigLoadError` on `Err`
    pub fn collect_imports(
        &mut self,
        interner: &mut Intern,
    ) -> (Option<Bind>, Vec<Import>, SourceDiagnosticSummary) {
        let mut imports: Vec<Import> = Vec::new();
        let mut bind: Option<Bind> = None;

        loop {
            self.skip_until_important();

            if self.cursor.pos() >= self.cursor.len() && self.peek() == b'\0' {
                break;
            }

            let ch = self.peek();

            match ch {
                b'"' => {
                    self.skip_quotes();
                }
                b'/' => {
                    if self.peek_ahead(1) == b'/' {
                        self.cursor.skip(2);
                        self.handle_comment();
                    } else if self.peek_ahead(1) == b'*' {
                        self.cursor.skip(2);
                        self.handle_multi_comment();
                    } else {
                        self.cursor.skip(1);
                    }
                }
                // Should probably just work with &str directly at this point
                c if c == b'i' || c == b'b' => {
                    // The operation requires a full utf-8 check
                    let prev_ch = self.peek_behind_char(1);
                    // WARN: band-aid :(
                    // The real fix to this is to make the parser more capable, which is not allowed. The
                    // parser stays this way for a large set of unexplained reasons that may change.
                    let can_check = prev_ch.is_whitespace()
                        // Start should always be valid
                        || self.cursor.pos() == 0
                        // Same as checking for start of file
                        || (self.cursor.pos() == keywords::EMBEDDING_CLAUSE_SIZE
                            // Is + 1 because we haven't actually advanced
                        && &self.cursor.items[0..keywords::EMBEDDING_CLAUSE_SIZE]
                            == b"@def");

                    if can_check {
                        if c == b'i' && self.is_import() {
                            let import = match self.parse_import(interner) {
                                Ok(i) => i,
                                Err(d) => {
                                    self.summary.push_diag(d);
                                    continue;
                                }
                            };

                            imports.push(import);
                        } else if c == b'b' && self.is_bind() {
                            bind = match self.parse_bind(interner) {
                                Ok(b) => Some(b),
                                Err(d) => {
                                    self.summary.push_diag(d);
                                    continue;
                                }
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

    /// Assumes the starting point is at the start quote
    fn parse_import(&mut self, interner: &mut Intern) -> Result<Import, SourceDiagnostic> {
        self.cursor.skip(1);
        let start_cursor = self.cursor.pos();
        // Boolean to track if a "\" was seen since only "/" can be used to separate
        let mut saw_backslash = false;

        while self.cursor.pos() < self.cursor.len() {
            match self.peek() {
                b'"' => {
                    self.cursor.skip(1);
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
                self.current_region.path_id,
            )
            .add_annotation(path_span, AnnotationKind::Primary, None)
            .build();

            return Err(src_diag);
        }

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
                    self.current_region.path_id,
                )
                .add_annotation(path_span, AnnotationKind::Primary, None)
                .build();

                return Err(src_diag);
            }
        };

        // We could check for an alias here too in case the file name is invalid and needs an alias
        //TODO:
        let file_name = match import_path.file_prefix().map(|n| n.to_str()).flatten() {
            Some(n) => n,
            _ => {
                let core_msg = format!(
                    "Failed to extract file name for path \"{}\"",
                    import_path.display()
                );

                //TODO: Aliasing
                let src_diag = SourceDiagnostic::builder(
                    ErrorCode::ImportErr.into(),
                    DiagnosticLevel::Error,
                    core_msg,
                    self.current_region.path_id,
                )
                .add_annotation(path_span, AnnotationKind::Primary, None)
                .build();

                return Err(src_diag);
            }
        };

        let name_id = interner.intern_str(&file_name);
        let path_id = interner.intern_path(&import_path);

        let alias_id: Option<SpannedContainer<InternedId>> = if self.is_as() {
            self.skip_whitespace();
            let start = self.cursor.pos() as u32;
            let id = self.read_id(interner);
            SpannedContainer::new(
                id,
                SourceSpan::new(self.current_region.self_id, start, self.cursor.pos() as u32),
            )
            .into()
        } else {
            None
        };

        let import_kind = ImportKind::UnresolvedSource(SpannedContainer::new(path_id, path_span));
        let import = Import::new(name_id, import_kind, alias_id);

        self.cfg.logger().log_dbg(|| {
            let import_name = interner.extract_str_from_interned_id(name_id);
            let region_path = interner.extract_path_from_path_id(self.current_region.path_id);
            format!(
                "Created import `{import_name}` (region=\"{}\")",
                region_path.display()
            )
        });

        Ok(import)
    }

    //WARN: Will be placed in different module
    //FIX: ugf3 8
    fn create_pathbuf(&self, slice: &[u8]) -> Result<PathBuf, SourceDiagnostic> {
        if cfg!(unix) {
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                let os_str = OsStr::from_bytes(slice);
                return Ok(PathBuf::from(os_str));
            }
        } else if cfg!(windows) {
            // NOTE: This may be done differently but remains a basic utf-8 check for now
            // #[cfg(windows)]
            // {
            //     use std::os::windows::ffi::OsStrExt;
            //     let os_str = OsStr::from_wide(slice);
            //
            //     return Ok(PathBuf::from(slice));
            // }
            //TODO: Do not. Enforce. UTF-8. !
            match str::from_utf8(slice) {
                Ok(s) => return Ok(PathBuf::from_str(&s).expect("Infallable")),
                Err(_) => {
                    let msg = "Invalid UTF-8 found within file".to_string();
                    let src_diag = SourceDiagnostic::builder(
                        None,
                        DiagnosticLevel::Error,
                        msg,
                        self.current_region.path_id,
                    )
                    .build();

                    return Err(src_diag);
                }
            }
        }

        match str::from_utf8(slice) {
            Ok(s) => Ok(PathBuf::from_str(&s).expect("Infallible")),
            Err(_) => {
                let msg = "Invalid UTF-8 found within file".to_string();
                let src_diag = SourceDiagnostic::builder(
                    None,
                    DiagnosticLevel::Error,
                    msg,
                    self.current_region.path_id,
                )
                .build();

                Err(src_diag)
            }
        }
    }

    fn parse_bind(&mut self, interner: &mut Intern) -> Result<Bind, SourceDiagnostic> {
        // skipping "
        self.cursor.skip(1);
        let start_cursor = self.cursor.pos();

        let mut saw_backslash = false;

        while self.cursor.pos() < self.cursor.len() {
            match self.peek() {
                b'"' => {
                    self.cursor.skip(1);
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
                self.current_region.path_id,
            )
            .add_annotation(path_span, AnnotationKind::Primary, None)
            .build();

            return Err(src_diag);
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
                    self.current_region.path_id,
                )
                .add_annotation(path_span, AnnotationKind::Primary, None)
                .build();

                return Err(src_diag);
            }
        };

        let bind_path_id = interner.intern_path(&bind_path);
        let bind = Bind::new(bind_path_id, path_span);

        self.cfg.logger().log_dbg(|| {
            let bind_path = interner.extract_path_from_path_id(bind_path_id);
            let region_path = interner.extract_path_from_path_id(self.current_region.path_id);
            format!(
                "Created bind which points to `{}` for region \"{}\"",
                bind_path.display(),
                region_path.display()
            )
        });

        Ok(bind)
    }

    fn read_id(&mut self, interner: &mut Intern) -> InternedId {
        let start = self.cursor.pos();

        while (self.cursor.pos() < self.cursor.len() && self.peek_char().is_alphanumeric())
            || (self.cursor.pos() < self.cursor.len() && self.peek() == b'_')
        {
            self.advance_char();
        }

        let end = self.cursor.pos();

        // Enforces utf-8 but module paths themselves don't need to be valid utf-8, am I
        // hallucinating?
        // Yes, yes.
        let id_str = str::from_utf8(&self.cursor.items[start..end])
            .expect("Cannot fail due to loop only accepting valid UTF-8 characters.");

        interner.intern_str(&id_str)
    }
    //FIX: Perf perf perf perf perfff

    fn peek_behind_char(&self, dest: usize) -> char {
        // Inclusive since otherwise it would skip the current character and there would need to be
        // a saturating sub to make up for it
        let chunk = &self.cursor.items[0..=self.cursor.pos()];

        //TODO: Fix forced validation
        // This is a bug. Call it a bug.
        // BUG: <- Bug
        std::str::from_utf8(chunk)
            .ok()
            .and_then(|c| c.chars().rev().skip(dest).next())
            .unwrap_or('\0')
    }

    fn peek_behind(&self, dest: usize) -> u8 {
        assert!(dest <= self.cursor.pos(), "cursor position underflow");
        self.cursor.peek_behind_owned_checked(dest).unwrap_or(b'\0')
    }

    fn skip_quotes(&mut self) {
        while self.cursor.pos() < self.cursor.len() {
            match self.peek() {
                b'\\' => {
                    self.cursor.skip(1);

                    self.read_escape();
                }
                b'"' => {
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

        //FIX: Utf 98
        self.skip_whitespace();

        if self.peek() != b'"' {
            return false;
        }

        true
    }

    // WARN: Suspicipus
    fn is_as(&mut self) -> bool {
        self.skip_whitespace();

        if self.cursor.pos() + 2 < self.cursor.len()
            && &self.cursor.items[self.cursor.pos()..self.cursor.pos() + 2] == b"as"
        {
            self.cursor.skip(2);
            return true;
        }

        false
    }

    fn is_bind(&mut self) -> bool {
        let start = self.cursor.pos();

        while self.cursor.pos() < self.cursor.len() && self.peek().is_ascii_alphabetic() {
            self.cursor.skip(1);
        }

        let end = self.cursor.pos();

        if &self.cursor.items[start..end] != b"bind" {
            return false;
        }

        //FIX: Utf 98
        self.skip_whitespace();

        if self.peek() != b'"' {
            return false;
        }

        true
    }

    fn handle_comment(&mut self) {
        while self.cursor.pos() < self.cursor.len() && self.peek() != b'\n' {
            self.cursor.skip(1);
        }
    }

    fn handle_multi_comment(&mut self) {
        let mut depth = 1;

        while self.cursor.pos() < self.cursor.len() && depth > 0 {
            if self.peek() == b'/' && self.peek_ahead(1) == b'*' {
                self.cursor.skip(1);
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

        let chunk = &self.cursor.items[self.cursor.pos()..];

        // Should be a test for this this is suspicious
        // Lazy evaluation to avoid utf-8 checking entire self.bytes
        std::str::from_utf8(chunk)
            .ok()
            .and_then(|c| c.chars().next())
            .unwrap_or('\0')
    }

    fn advance_char(&mut self) -> char {
        let ch = self.peek_char();
        self.cursor.skip(ch.len_utf8());
        ch
    }

    fn peek_ahead(&self, dest: usize) -> u8 {
        self.cursor.peek_ahead_owned_checked(dest).unwrap_or(b'\0')
    }

    fn skip_until_important(&mut self) {
        // Stopping at parts that may cause wrongful import reads
        while self.cursor.pos() <= self.cursor.len()
            && self.peek() != b'i'
            && self.peek() != b'b'
            && self.peek() != b'"'
            && self.peek() != b'/'
        {
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
