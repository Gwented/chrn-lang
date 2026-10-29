// 65540 SHOULD be the maximum amount of bytes, given that lookahead wise at most the buffer will
// allow lookahead. Or not. Depends.
//! This module represents the stage of `chrn` processing where there it may read an entire file, or
//! it may read between `@def` and `@end`. This exists so that if there is serial data within the
//! file, the entire file isn't forced to be loaded into memory, which would be a net negative.
//!
//! IMPORTANT NOTES:
//! This loader does NOT UTF-8 check anything, it merely ensures that the region is valid for chrn's
//! language rules. Only bytes are operated upon. The diagnostics sent expect that users of this
//! data check that the data is valid and reacts accordingly.

//! Uses a configurable buffer that grows-..
pub mod config_loader_summary;
//TODO: Need relative spanning in renderers
use std::{
    io::{BufRead, BufReader, Read},
    ops::Range,
};

use chrn_utils::{
    core_error::ConfigLoadError,
    err_codes::ErrorCode,
    id_types::{PathId, SourceRegionId},
    source_map::{
        source_diagnostic::{
            DiagnosticLevel, SourceDiagnostic, SourceDiagnosticSink, SourceDiagnosticSummary,
            annotations::AnnotationKind,
        },
        source_region::SourceRegion,
        source_span::SourceSpan,
    },
    utils::FreezeTrackerU32,
};

use lang::keywords::EMBEDDING_CLAUSE_SIZE;

use crate::chrn_config::ChrnConfig;

/// Can read 64KB before stopping if no `@def` or EOF is found
/// This exists because the maximum searchable size does not imply that the region size aligns with
/// said size.
const MAX_SEARCH_READ: usize = 1024 * 64;

//WARN: Feels like this should error earlier on rather than just silently defaulting to 1 byte
//reads.
/// Minimum size the buffer can be set to.
const MIN_BUFFER_SIZE: usize = 1;

//FIX: Change to 64Kib since this will be a choice so now it'sj ust default 64Kib I'm in a vehicle.
//I'm not in a vehicle. Maybe we can go for 128Kib directly since that is literally nothing.
//Using 32Kib to ensure smalelr sizes workrkkrk
/// ~40 Kib buffer
const DEFAULT_BUFFER_SIZE: usize = (128 * 1024) / 3;
// const DEFAULT_BUFFER_SIZE: usize = 1;

// Can read at most 32 KB before an @def, can read at most 32 KB after an @def

//Call this RegionLoader?
//TODO: A script start should dictate how many bytes can be read.
//So, if script_start is 2048, total_buffer is now starting at byte 2048 of handle, and it has 16KB
//from that point forward.
pub struct ConfigLoader<'a, R: Read> {
    // Since region ids are not used by the loader for diagnostics, this is safe. Only the path id
    // is used.
    current_region_id: SourceRegionId,
    current_path_id: PathId,
    cfg: &'a ChrnConfig,
    ln_num_tracker: FreezeTrackerU32,
    col_tracker: FreezeTrackerU32,
    /// Position specifically used for navigating the IO buffer
    cursor: usize,
    abs_pos: usize,
    /// The max amount of bytes to stop it, which is dynamically set, hence why it's apart of the struct.
    /// (It says dynamically set but it's not dynamically set yet. It just relies on
    /// `bytes_consumed_rel`)
    limit: usize,
    // Need to keep script_start to have this be a method where we have the abs version by default.
    /// Bytes consumed relative to `@def` if present, otherwise it's relative to the start of file,
    /// 0.
    bytes_consumed_start_rel: usize,
    // -- NEW --
    /// Buffer that may or may not exceed 64 Kib which is where operations should index from.
    persistent_buffer: Vec<u8>,
    /// Buffer that refills `persistent_buffer`. This should never be indexed.
    handle: BufReader<R>,
    /// The absolute start of the file which may be simply pos = 0 or starting at the d in @def
    script_start: usize,
}

//TODO: Persistent buffers need to store. We need a writing pos.

// At most will read

// Gorf
/// This exists as a local output return, which may or may not hold an error that is local to
/// this stage since the caller cares about and uses the region, but the external caller
/// (like a CLI) only wants the specific `ConfigLoaderError` that may exist.
#[derive(Debug)]
pub enum ConfigLoaderOutput {
    // Should this just be Option ConfigLoaderError?
    /// Loaded region with no issues
    Success(SourceRegion, SourceDiagnosticSummary),
    /// A region that experienced an error during the loading process.
    ///
    /// This is not called "partial" because an error like `@def` without `@end` isn't a partial
    /// viewing, it's just a broken viewing.
    Broken(SourceRegion, ConfigLoadError),
    /// A region that at some point got an error to where
    /// Loading failed too early so no part of the region was read.
    UnrecoverableErr(ConfigLoadError),
}

//NOTE: This forces paths to be given, but if the chrn file itself doesn't have a path given
//then the language doesn't work anyways. May leave as is.
impl<R: Read> ConfigLoader<'_, R> {
    /// Uses `PathId` for error location reporting purposes
    pub fn new<'a>(
        current_region_id: SourceRegionId,
        handle: R,
        current_path_id: PathId,
        cfg: &'a ChrnConfig,
    ) -> ConfigLoader<'a, R> {
        Self::with_capacity(
            current_region_id,
            handle,
            current_path_id,
            DEFAULT_BUFFER_SIZE,
            cfg,
        )
    }

    /// Uses `PathId` for error location reporting purposes
    pub fn with_capacity<'a>(
        current_region_id: SourceRegionId,
        handle: R,
        current_path_id: PathId,
        buf_cap: usize,
        cfg: &'a ChrnConfig,
    ) -> ConfigLoader<'a, R> {
        ConfigLoader {
            current_region_id,
            cfg,
            current_path_id,
            handle: BufReader::with_capacity(buf_cap.max(MIN_BUFFER_SIZE), handle),
            col_tracker: FreezeTrackerU32::new(1),
            ln_num_tracker: FreezeTrackerU32::new(1),
            cursor: 0,
            abs_pos: 0,
            limit: MAX_SEARCH_READ,
            bytes_consumed_start_rel: 0,
            persistent_buffer: Vec::new(),
            script_start: 0,
        }
    }

    /// Returns a Success value of the bytes to Lex, the offset of where to start lexing if an
    /// `@def` is present, and the offset of where to start reading the serialized data if an
    /// `@def` and `@end` is present. Returns a `ConfigLoadError` upon failure that has internal
    /// error details.
    pub fn load_config(&mut self) -> ConfigLoaderOutput {
        // Doesn't NEED definition but will error if declared and not closed
        let mut requires_end = false;

        // Setup to allow for unclosed quotes to be tracked since this stage is very sensitive to
        // such errors.
        let mut first_double_quote = None;
        let mut first_single_quote = None;

        let mut double_quotes_seen = 0;
        let mut single_quotes_seen = 0;

        let mut def_span: Option<SourceSpan> = None;

        // This is here because technically, a warn CAN be emitted, and should be kept track of.
        // But it's not owned by the loader itself because the loader can only emit one error
        // at a time all of them are terminal
        let mut diag_summary = SourceDiagnosticSummary::default();

        loop {
            let b = match self.peek() {
                Ok(b_opt) => match b_opt {
                    Some(b) => b,
                    None => break,
                },
                Err(e) => {
                    //TODO:
                    let out = if self.persistent_buffer.is_empty() {
                        let cfg_err = ConfigLoadError::IO(e);
                        ConfigLoaderOutput::UnrecoverableErr(cfg_err)
                    } else {
                        let broken_region = self.create_region(None, !requires_end);
                        let cfg_err = ConfigLoadError::IO(e);
                        ConfigLoaderOutput::Broken(broken_region, cfg_err)
                    };
                    return out;
                }
            };

            let span_start = self.cursor;

            match b {
                // May reduce duplication by using mini-state that keeps track of quotes but fine
                // for now
                b'"' => {
                    let quote_start = self.cursor;
                    // Confirmed by match arm
                    let Ok(Some(quote_type)) = self.advance() else {
                        unreachable!()
                    };

                    double_quotes_seen += 1;

                    // Is there a reason for lines_read to be printed if there are multiple quotes?
                    // When are there ever NOT multiple quotes if it's in a serialized file?
                    match self.read_quotes(quote_type) {
                        QuoteResult::Success => (),
                        QuoteResult::Unclosed => {
                            let core_msg = "Unclosed quotes reached end of read limit".to_string();

                            let rel_q_start = quote_start as u32;
                            let q_span = SourceSpan::new(
                                self.current_region_id,
                                rel_q_start,
                                rel_q_start + 1,
                            );

                            let mut builder = SourceDiagnostic::builder(
                                None,
                                DiagnosticLevel::Error,
                                core_msg,
                                self.current_path_id,
                            )
                            .add_annotation(
                                q_span,
                                AnnotationKind::Primary,
                                "Possible start of unclosed quote".to_string().into(),
                            );

                            if double_quotes_seen > 1 {
                                let note = "There are other quotes within the file so the line given could be incorrect".to_string();
                                builder = builder.add_note(note);
                            };

                            let src_diag = builder.build();
                            let broken_region = self.create_region(None, !requires_end);

                            return ConfigLoaderOutput::Broken(
                                broken_region,
                                ConfigLoadError::Diagnostic(src_diag),
                            );
                        }
                        QuoteResult::IO(e) => {
                            let broken_region = self.create_region(None, !requires_end);
                            let config_load_err = ConfigLoadError::IO(e);
                            return ConfigLoaderOutput::Broken(broken_region, config_load_err);
                        }
                    }

                    if double_quotes_seen == 1 {
                        first_double_quote = Some(quote_start)
                    };
                }
                b'\'' => {
                    let quote_start = self.cursor;
                    let Ok(Some(quote_type)) = self.advance() else {
                        // Confirmed by match arm
                        unreachable!()
                    };

                    single_quotes_seen += 1;

                    match self.read_quotes(quote_type) {
                        QuoteResult::Success => (),
                        QuoteResult::Unclosed => {
                            let core_msg = "Unclosed quotes reached end of read limit";

                            let rel_q_start = quote_start as u32;
                            let q_span = SourceSpan::new(
                                self.current_region_id,
                                rel_q_start,
                                rel_q_start + 1,
                            );

                            let mut diag_builder = SourceDiagnostic::builder(
                                None,
                                DiagnosticLevel::Error,
                                core_msg,
                                self.current_path_id,
                            )
                            .add_annotation(
                                q_span,
                                AnnotationKind::Primary,
                                "Possible start of unclosed quote".to_string().into(),
                            );

                            if single_quotes_seen > 1 {
                                let note = "There are other quotes within the file so the line given could be incorrect".to_string();
                                diag_builder = diag_builder.add_note(note);
                            };

                            let src_diag = diag_builder.build();

                            let broken_region = self.create_region(None, !requires_end);

                            return ConfigLoaderOutput::Broken(
                                broken_region,
                                ConfigLoadError::Diagnostic(src_diag),
                            );
                        }
                        QuoteResult::IO(e) => {
                            let broken_region = self.create_region(None, !requires_end);
                            let cfg_err = ConfigLoadError::IO(e);
                            return ConfigLoaderOutput::Broken(broken_region, cfg_err);
                        }
                    }

                    if single_quotes_seen == 1 {
                        first_single_quote = Some(quote_start)
                    };
                }
                b'/' => {
                    // Should there be an advance unchecked? For scenarios like this we already have
                    // the byte in memory making it impossible to fail
                    _ = self.advance();

                    let current_byte = match self.peek() {
                        Ok(b_opt) => match b_opt {
                            Some(b) => b,
                            None => break,
                        },
                        Err(e) => {
                            let broken_region = self.create_region(None, !requires_end);
                            let cfg_err = ConfigLoadError::IO(e);
                            return ConfigLoaderOutput::Broken(broken_region, cfg_err);
                        }
                    };

                    if current_byte == b'/' {
                        _ = self.advance();
                        match self.handle_comment() {
                            Ok(_) => (),
                            Err(e) => {
                                let broken_region = self.create_region(None, !requires_end);
                                let config_load_err = ConfigLoadError::IO(e);
                                return ConfigLoaderOutput::Broken(broken_region, config_load_err);
                            }
                        }
                    } else if current_byte == b'*' {
                        _ = self.advance();
                        match self.handle_multi_comment() {
                            MultiCommentResult::Success => (),
                            MultiCommentResult::Diagnostic(diag) => {
                                let region = self.create_region(None, !requires_end);
                                let cfg_err = ConfigLoadError::Diagnostic(diag);
                                return ConfigLoaderOutput::Broken(region, cfg_err);
                            }
                            MultiCommentResult::IO(e) => {
                                let region = self.create_region(None, !requires_end);
                                let cfg_err = ConfigLoadError::IO(e);
                                return ConfigLoaderOutput::Broken(region, cfg_err);
                            }
                        }
                    }
                }
                b'@' => {
                    // This @ is not skipped for the sake of keeping self.pos at the same starting point.

                    let clause_end_abs = self.abs_pos + (EMBEDDING_CLAUSE_SIZE - 1);
                    // The limit is relative to 32Kib so we need a rel version

                    // NOTE: Why is this - 1 here?
                    // I think it was something related to since self.cursor is at @, and embedding
                    // clause is @ + def, not using -1 technically means we're looking at @defn
                    // where n is a random byte. EMBEDDING_CLAUSE_SIZE encodes '@' but we are
                    // already at '@' meaning we need to cut off the extra '@'.
                    // Meaning, it's going from @ + @def to @ + def.
                    let clause_end_pos = self.cursor + (EMBEDDING_CLAUSE_SIZE - 1);

                    //TODO: Help msg for if an @ was seen but it would exceed size
                    // Helper boolean
                    //
                    // Possible source of indexing bug could be that ANNOTATION_CLAUSE_SIZE, which
                    // is sliced exclusively here, would exceed the length since it didn't ensure
                    // it's length was inside the constraints of the buffer.
                    //
                    // This is hard to track since the overwhelming majority of the time, there's at
                    // least a '\n' after @def and @end, making most operations succeed anyways
                    // since it's impossible to get the error unless the environment specifically
                    // does not have at most one extra byte
                    //
                    // Checks if < self.limit to prevent over-stepping 32Kib limit

                    //NOTE: If we are at say 1 byte below 65536 (64Kib) @def/@end readings will NOT
                    //attempt to read past the limit. This IS possible and would not break the
                    //region limit but keeping that out for simplicity.

                    //FIX: "cfg_at_end_straddling_read_limit_is_rejected_test" fails because @end
                    //extends past length. Need to say "can_check" is false
                    let maybe_embed_slice =
                        // Since this is @ + @def, it ends one byte after @def, which is what we
                        // want since it's exclusive.
                        match self.slice(self.cursor..self.cursor + EMBEDDING_CLAUSE_SIZE) {
                            Ok(sl_opt) => match sl_opt {
                                Some(sl) => sl,
                                None => break,
                            },
                            Err(e) => {
                                let broken_region = self.create_region(None, !requires_end);
                                let config_load_err = ConfigLoadError::IO(e);
                                return ConfigLoaderOutput::Broken(broken_region, config_load_err);
                            }
                        };

                    let is_def = maybe_embed_slice == b"@def";
                    let is_end = maybe_embed_slice == b"@end";

                    // Read this as claude 4 different times. I can't do this anymore.
                    let can_check = clause_end_pos < self.limit;

                    // -- OLD BEHAVIOR THAT MAY BE RE-APPLIED --
                    // If `@def` was seen, there is enough space to check, and `@end` aligns with
                    // the bytes seen, set the serial start to the position 1 after `@end`
                    // -- OLD BEHAVIOR THAT MAY BE RE-APPLIED --
                    //
                    // This no longer requires @def first for now, so files can end with @end
                    // without any `@def` start syntax.
                    //
                    // Probably should not be allowed since the behavior from this is fairly
                    // non-deterministic unless someone is actively counting bytes
                    if can_check && is_end {
                        // Starts exactly 1 byte after "@end"
                        // This variable is proof there was a script block, but does not imply
                        // serial data. This is more like "script_end" where script_end is where
                        // serial starts.
                        let serial_start_rel = self.cursor + EMBEDDING_CLAUSE_SIZE;
                        // Doesn't use the tracked abs pos because we do not update it before
                        // entering this condition.
                        let serial_start_abs = serial_start_rel + self.script_start;

                        // If doesn't require end then that means this is an `@end` only block,
                        // which needs to start at the start of the file line tracking-wise
                        let needs_reset = !requires_end;

                        if needs_reset {
                            self.ln_num_tracker.reset_soft();
                            self.col_tracker.reset_soft();
                        }

                        //WARN: Doesn't use helper because the helper doesn't take in a position
                        //argument. It doesn't take one because it doesn't seem meaningful, it seems
                        //like a forced compatibility later for this one singular case.
                        let region = SourceRegion::new(
                            self.ln_num_tracker.val(),
                            self.col_tracker.val(),
                            self.persistent_buffer[..serial_start_rel].to_vec(),
                            self.current_region_id,
                            self.current_path_id,
                            self.script_start,
                            Some(serial_start_abs),
                        );
                        debug_assert!(
                            region.src_bytes.len() <= chrn_utils::MAX_REGION_SIZE,
                            "Max region size exceeded [Found {}]",
                            region.src_bytes.len()
                        );

                        return ConfigLoaderOutput::Success(region, diag_summary);
                    }

                    // If no @def has been seen yet, there is enough space to check, and the next 4
                    // bytes are "@def", set a requirement for an "@end", set the `script_start` to
                    // the "@" in "@def", skip "@def", then set the Option def_span to the current
                    // position span

                    if !requires_end && can_check && is_def {
                        requires_end = true;
                        self.script_start = self.cursor;

                        // Can we derive?
                        // Resets so that the max region size can be properly accounted for
                        self.bytes_consumed_start_rel = 0;
                        // The last buffer may or may not have needed to push more than 64Kib to
                        // peek, slice, etc., so this carrires [@def..] to new buffer to corruption.
                        let carry_over = self.persistent_buffer[self.cursor..].to_vec();
                        self.persistent_buffer = carry_over;

                        // Positioning to the relative byte after @def.
                        self.cursor = EMBEDDING_CLAUSE_SIZE;

                        self.ln_num_tracker.freeze();
                        self.col_tracker.freeze();

                        // This needs to be relative since only regions are used
                        // This is safe to hard-code because the condition itself only allows for
                        // this to be made if it's the first time @def is seen, which has to be the
                        // beginning of the region.
                        let rel_start = 0;
                        let rel_end = 4;
                        def_span =
                            Some(SourceSpan::new(self.current_region_id, rel_start, rel_end));

                        // Needs to continue or the last advance causes one-off errors since it
                        // conflicts with @def which already deals with itself
                        continue;
                    }

                    // This should not fail since if `@def` and `@end` fail, it should not consume
                    // anything in the process and just treat this as though it were a singular @ to
                    // skip
                    debug_assert_eq!(self.persistent_buffer[self.cursor], b'@');
                    _ = self.advance();
                }
                _ => {
                    _ = self.advance();
                }
            }
        }
        // TODO: Assert this...

        //TEST: For consuming rest if there was a `break` before reading up to limit
        // This allows for error msgs to look less confusing
        while let Ok(Some(_)) = self.advance() {}

        let region = self.create_region(None, false);
        debug_assert!(
            region.src_bytes.len() <= chrn_utils::MAX_REGION_SIZE,
            "Max region size exceeded"
        );

        // If the loop was broken because the limit was reached, and there is a byte after the
        // limit, that means it stopped because it reached the max read bytes not because of a valid
        // script file. So if limit + 1 == Some then probably an error
        //
        // Does not conflict with @end since @end has it's own return environment.
        //
        // NOTE: Needs direct indexing because peek already reached it's limit
        if self.peek_no_limit().is_ok_and(|b| b.is_some()) {
            // Sole reason this is here
            let core_msg = "Amount of bytes in range exceeds max of 64KiB";
            let diag = SourceDiagnostic::builder(
                ErrorCode::CompilerInternals.into(),
                DiagnosticLevel::Warn,
                core_msg,
                self.current_path_id,
            )
            //WARN: Maybe change depending on if @def WAS actually used
            .add_note("Possibly missing @def or @end usage")
            .build();
            diag_summary.push_diag(diag);
        }

        // Case of no @def and no @end which requires a 'None' return from the main loop.
        // This does not mean it is correct, it only means the read limit wasn't reached before the
        // file ended.
        if !requires_end {
            ConfigLoaderOutput::Success(region, diag_summary)
        } else {
            // Case of end <eof> being reached, which means it is within `READ_LIMIT` since it
            // didn't actively reach it
            let core_msg = "Could not find `@end` after `@def`";

            let def_span = def_span.expect("@def must exist for this branch to be seen");
            // Explicitly declaring this so the end of the file can be pointed at too

            // Ensuring an indexable character is the last character
            //WARN: Need to make sure this doesn't break things
            // WARN: ENSURE THIS DOES NOT BREAK ANYTHING

            // (inclusive, exclusive) end
            //
            //  If we have "text<eof>", it advances "t" stopping at "<eof>", which naturally fits
            //  exclusive since eof is just a byte position that will never be touched
            //
            //  cursor cannot exceed 64Kib so it is treated as the eof pos, rather than
            //  persistent_buffer.len()
            let eof_pos = self.cursor as u32;

            // Need to - 1 so that the spanning doesn't have a len of 0.
            // Cannot do + 1 to the end of the span or it extends one past len
            let eof_span = SourceSpan::new(self.current_region_id, eof_pos - 1, eof_pos);

            let src_diag = SourceDiagnostic::builder(
                ErrorCode::ConfigLoadErr.into(),
                DiagnosticLevel::Error,
                core_msg,
                self.current_path_id,
            )
            .add_annotation(
                def_span,
                AnnotationKind::Secondary,
                "`@def` started here".to_string().into(),
            )
            .add_annotation(
                eof_span,
                AnnotationKind::Primary,
                "Unexpected <eof>".to_string().into(),
            )
            .build();

            ConfigLoaderOutput::Broken(region, ConfigLoadError::Diagnostic(src_diag))
        }
    }

    // What a useful bit of documentation that describes the exact enum present
    // Also this should be encoded differently but not sure if we really want to start making domain
    // specific enums per method.
    /// Returns Ok(`true`) on success, Ok(`false`) on unclosed quotes (Maybe?)
    /// Returns `Err` upon io error
    // TODO: LEXER SHOULD ALSO HANDLE THIS ALONE
    fn read_quotes(&mut self, quote_type: u8) -> QuoteResult {
        loop {
            let b = match self.peek() {
                Ok(b_opt) => match b_opt {
                    Some(b) => b,
                    None => return QuoteResult::Unclosed,
                },
                Err(e) => return QuoteResult::IO(e),
            };

            match b {
                b'\\' => {
                    // If this isn't checked for then in the scenario "Hello\" it'll go past eof
                    // since it's assuming something is being escaped.

                    let fully_skipped = match self.skip(2) {
                        Ok(res) => res,
                        Err(e) => return QuoteResult::IO(e),
                    };

                    if !fully_skipped {
                        return QuoteResult::Unclosed;
                    }
                }
                b if b == quote_type => {
                    _ = self.advance();
                    return QuoteResult::Success;
                }
                _ => {
                    _ = self.advance();
                }
            }
        }
    }

    /// Helper for reducing region creation boiler-plate
    fn create_region(
        &mut self,
        serial_start_opt: Option<usize>,
        needs_reset: bool,
    ) -> SourceRegion {
        // Resetting since this means there was no `@def` or `@end`
        // This needs to be done because the counter would otherwise remain at the last line and
        // col, rather than sitting at the first.
        if needs_reset {
            self.ln_num_tracker.reset_soft();
            self.col_tracker.reset_soft();
        }

        //WARN: This is dangerous because it could hide bugs, but it isn't necessarily wrong.
        //Maybe it shouldn't be here though.
        //
        // For cases like broken regions there either needs to be a clamp or a complex system where
        // the max length of persistent_buffer shrinks when needed.
        let end = self.cursor.min(chrn_utils::MAX_REGION_SIZE);
        SourceRegion::new(
            self.ln_num_tracker.val(),
            self.col_tracker.val(),
            self.persistent_buffer[..end].to_vec(),
            self.current_region_id,
            self.current_path_id,
            self.script_start,
            serial_start_opt,
        )
    }

    fn handle_comment(&mut self) -> Result<(), std::io::Error> {
        // Should this skip \n itself? Does it matter? (No)
        while let Some(b) = self.peek()?
            && b != b'\n'
        {
            _ = self.advance();
        }
        Ok(())
    }

    /// Expects to start at the 'n' in "/*n",
    fn handle_multi_comment(&mut self) -> MultiCommentResult {
        let mut depth = 1;

        //WARN: DOES THIS NEED RELATIVE?
        // To adjust multi-comment start to the first '/'. /*c - 2 = /
        let comment_start_rel = (self.cursor - 2) as u32;
        while depth > 0 {
            let current_byte = match self.peek() {
                Ok(b_opt) => match b_opt {
                    Some(b) => b,
                    None => break,
                },
                Err(e) => return MultiCommentResult::IO(e),
            };

            let next_byte = match self.peek_ahead(1) {
                Ok(b_opt) => match b_opt {
                    Some(b) => b,
                    None => {
                        // Since the looe is just a peek we need to make we need to advance here too
                        _ = self.advance();
                        break;
                    }
                },
                Err(e) => return MultiCommentResult::IO(e),
            };

            if current_byte == b'/' && next_byte == b'*' {
                depth += 1;
                self.skip_unchecked(2);
            } else if current_byte == b'*' && next_byte == b'/' {
                depth -= 1;
                self.skip_unchecked(2);
            } else {
                self.skip_unchecked(1);
            }
        }

        //FIXME: Definitely broken
        if depth > 0 {
            let core_msg = "Unclosed multi-line comment";

            // To include full multi-line syntax. / + 2 = /*
            let comment_start_span = SourceSpan::new(
                self.current_region_id,
                comment_start_rel,
                comment_start_rel + 2,
            );

            // Since the length of the buffer may or may not be > MAX_REGION_SIZE we need to
            // clamp it. The buffer itself does not promise to be < MAX_REGION_SIZE
            let end = (self
                .persistent_buffer
                .len()
                .min(chrn_utils::MAX_REGION_SIZE)) as u32;

            let eof_span = SourceSpan::new(self.current_region_id, end - 1, end);

            let builder = SourceDiagnostic::builder(
                None,
                DiagnosticLevel::Error,
                core_msg,
                self.current_path_id,
            )
            .add_annotation(
                comment_start_span,
                AnnotationKind::Secondary,
                "Comment starts here".to_string().into(),
            )
            .add_annotation(
                eof_span,
                AnnotationKind::Primary,
                "Unexpected <eof>".to_string().into(),
            );

            // If a file really did hit eof during a multi-line comment the file is more likely than
            // not broken to even attempt to view. Also the lexer would get really really scared if
            // it had to deal with this.
            return MultiCommentResult::Diagnostic(builder.build());
        }

        MultiCommentResult::Success
    }

    //TODO: Maybe add a refill where it avoids advances entirely and instead just fills the
    //persistent buffer and reports if it couldn't reach the amount given.
    // What to return
    /// Returns Ok(`true`) if the full skip was done. Ok(`false`) if reached EOF before full skip.
    fn skip(&mut self, amt: usize) -> Result<bool, std::io::Error> {
        for _ in 0..amt {
            match self.advance()? {
                Some(b) => b,
                None => return Ok(false),
            };
        }
        debug_assert!(
            self.persistent_buffer
                .get(self.cursor - amt..self.cursor)
                .is_some(),
            "`skip` successfully advanced but did not append to buffer"
        );
        Ok(true)
    }

    // This skip operation has only be used safely in this context. It's only used in scenarios like
    // multi-comments where look-ahead was already done to know that 2 bytes at most exist.
    fn skip_unchecked(&mut self, amt: usize) {
        // Uses this so othat it can re-use `advance()` since it still needs to '\n' and col track
        for _ in 0..amt {
            _ = self.advance();
        }
    }

    fn advance(&mut self) -> Result<Option<u8>, std::io::Error> {
        //WARN: Check my semantics
        let Some(b) = self.peek()? else {
            return Ok(None);
        };
        if b == b'\n' {
            self.ln_num_tracker.increment();
            self.col_tracker.reset_soft();
        } else {
            self.col_tracker.increment();
        }
        self.cursor += 1;
        self.abs_pos += 1;
        self.bytes_consumed_start_rel += 1;

        Ok(Some(b))
    }

    fn peek_ahead(&mut self, amt: usize) -> Result<Option<u8>, std::io::Error> {
        // Include the byte being inspected in the exclusive end.
        if !self.refill_until(amt + 1)? {
            return Ok(None);
        }

        Ok(Some(self.persistent_buffer[self.cursor + amt]))
    }

    fn peek(&mut self) -> Result<Option<u8>, std::io::Error> {
        if self.bytes_consumed_start_rel == self.limit {
            return Ok(None);
        }
        self.try_refill()?;

        Ok(self.persistent_buffer.get(self.cursor).copied())
    }

    // This is just for the final peek since we need to be able to see if we're on a byte past the
    // limit while maintaining the try_refill
    fn peek_no_limit(&mut self) -> Result<Option<u8>, std::io::Error> {
        self.try_refill()?;
        Ok(self.persistent_buffer.get(self.cursor).copied())
    }

    /// Ensures distance given is plaaaussifjblee
    /// This is a helper over skip
    /// Returns Ok(`true`) if the full skip was done. Ok(`false`) if reached EOF before full skip.
    fn ensure_ahead(&mut self, amt: usize) -> Result<bool, std::io::Error> {
        // let col = self.col_tracker.clone();
        // let initial_pos = self.cursor;
        // let ln_num = self.ln_num_tracker.clone();

        if (self.cursor + amt) >= self.persistent_buffer.len() {
            if !self.refill_until(amt + 1)? {
                return Ok(false);
            }
        }

        todo!()
    }

    //TEST: Would be mutable since we need to push to persistent buffer to actually continue
    //reading.
    /// Does a slice that does not advance the cursor, but may or may not grow the
    /// persistent buffer to form the desired slice.
    fn slice(&mut self, range: Range<usize>) -> Result<Option<&[u8]>, std::io::Error> {
        debug_assert_eq!(range.start, self.cursor);
        if !self.refill_until(range.len())? {
            return Ok(None);
        }

        let slice = &self.persistent_buffer[range];
        Ok(Some(slice))
    }

    /// Refills only if the entire buffer has been read
    fn try_refill(&mut self) -> Result<(), std::io::Error> {
        //NOTE: What about peek aheads? We need the thing where we refill and always ahead enough
        //where peek ahead is never an issue.
        if self.cursor < self.persistent_buffer.len() {
            return Ok(());
        }
        //TEST: New gimmick
        self.unchecked_refill()
    }

    // Could return something where it's {EOF, FullRefill(bool), IO} so that callers can choose to
    // return it question mark
    /// Returns Ok(`true`) if `amt` bytes starting at the cursor are available within the limit.
    /// Returns Ok(`false`) if the request exceeds the limit or EOF occurs before it.
    fn refill_until(&mut self, amt: usize) -> Result<bool, std::io::Error> {
        let end = self.cursor + amt;
        if end > self.limit {
            return Ok(false);
        }

        while self.persistent_buffer.len() < end {
            let last_len = self.persistent_buffer.len();
            self.unchecked_refill()?;
            if self.persistent_buffer.len() == last_len {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // Is unchecked a good term here? It's not dangerous it's just allocating more than currently
    // needed.
    /// Consumes current buffer then appends new buffer to `persistent_buffer`
    fn unchecked_refill(&mut self) -> Result<(), std::io::Error> {
        // Since persistent buffer is the src of truth the buffer solely supplies the persistent buffer.
        // It's the src of truth because peek_ahead needs a dependable source
        let buf_len = self.handle.buffer().len();
        self.handle.consume(buf_len);
        match self.handle.fill_buf() {
            Ok(buf) => {
                self.persistent_buffer.extend_from_slice(buf);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

enum MultiCommentResult {
    Success,
    Diagnostic(SourceDiagnostic),
    IO(std::io::Error),
}

enum QuoteResult {
    Success,
    Unclosed,
    IO(std::io::Error),
}
