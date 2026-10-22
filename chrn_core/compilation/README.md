## Add a directive

- Preload its name in `chrn_utils::intern` and update `INTERNER_PRELOAD_SIZE` as needed. Preserve existing IDs.
- Add its name to `lang::directive_consts::BUILTIN_DIRECTIVE_STRS`.
- For an inline directive, update `DirectiveInline` or `TypeDirective` and its `try_from_interned_str` lookup. Add its fixed index and `directive_to_id_const` mapping in `script_compiler/compiler_consts.rs`, then append the matching entry to `compiler_helpers::DIRECTIVES_DATASET`. `load_directives()` loads that dataset in index order.
- For a preprocess directive, add its `DirectivePreprocessSchemaKind`, schema, interned-ID lookup, and processing behavior. Set the stage at which its arguments can be processed. Preprocess directives are parameterized and do not occupy preloaded inline directive slots.
- Update the documentation and directive tests.
