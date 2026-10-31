# CLI

## Structured diagnostics

JSON and YAML diagnostics expose the stored source metadata:

- File sources have a `path` and a `null` `source_name`.
- In-memory sources have a `null` `path` and their optional `source_name`.
  Unnamed sources have both fields set to `null`.

Annotation spans use `region_source_name` and `region_path` from their own source.
Both fields are `null` when the source region is unavailable. Span `start` and `end` are byte offsets projected
through the source region when available.

## Extensions
Since the `chrn` base language is meant to be extended upon by using it's metadata from script files/blocks, the cli allows for extensions.

Any existent path variable with a binary executable that starts with "chrn-" will be executed.

Meaning, if there exists "chrn-json" and "chrn json" is typed, it will see that "chrn json" is not a valid argument and search for that binary matching the convention, which will lead to it being executed just like any other built-in argument.

To use this, there must exist a `CHRN_EXTENSIONS=1` path variable, otherwise it will not search for extensions. Extensions are not searched for by default due to possible arbitrary code execution with such a functionality.
