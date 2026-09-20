Config loader rejects any script block/file above 32 KiB in [config_loader](./compilation/src/config_loader.rs). If a file exceeds 32 Kib it is truncated and a warn is given.

Module amount can't exceed MAX_MODULES in [chrn_utils](./chrn_utils/src/lib.rs)
Max diagnostics are controlled by external tooling decisions through `Budget` usage in [source_diagnostic](./chrn_utils/src/source_map/source_diagnostic.rs)
Integers and floats are arbitrarily memory bounded, but `ChrnConfig`'s `max_numeric_bits` is what actually controls what bit range is allowed. By default, it abides by [`compilation::DEFAULT_MAX_NUMERIC_BITS`](./compilation/src/lib.rs) which is 256, but external tooling and src file chosen configuration can set their own.
The CLI accepts numeric limits up to 1,000,000 bits. Left shifts retain an unconditional 1,000,000-bit shift ceiling even when an embedding supplies a larger numeric limit.

Where possible, loops use `loop_abort!`, maybe convert this into a real error if this was more so a geniune attempt at overloading the system? Right now it just assumes this was an internal bug.
