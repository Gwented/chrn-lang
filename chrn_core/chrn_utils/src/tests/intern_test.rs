use std::path::Path;

use crate::{
    id_types::{
        InternedId, InternedIdUTF8, PathId,
        id_tags::{MaybeUTF8Tag, UTF8Tag},
    },
    intern::{self, Intern},
};

#[test]
fn keyword_intern_alignment() {
    let interner = Intern::init();

    assert_eq!(
        "self",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_SELF).into_tagged::<UTF8Tag>()),
        "INTERNED_SELF (0) should be 'self'"
    );
    assert_eq!(
        "struct",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_STRUCT).into_tagged::<UTF8Tag>()),
        "INTERNED_STRUCT (1) should be 'struct'"
    );
    assert_eq!(
        "import",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_IMPORT).into_tagged::<UTF8Tag>()),
        "INTERNED_IMPORT (2) should be 'import'"
    );
    assert_eq!(
        "export",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_EXPORT).into_tagged::<UTF8Tag>()),
        "INTERNED_EXPORT (3) should be 'export'"
    );
    assert_eq!(
        "bind",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_BIND).into_tagged::<UTF8Tag>()),
        "INTERNED_BIND (4) should be 'bind'"
    );
    assert_eq!(
        "alias",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_ALIAS).into_tagged::<UTF8Tag>()),
        "INTERNED_ALIAS (5) should be 'alias'"
    );
    assert_eq!(
        "let",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_LET).into_tagged::<UTF8Tag>()),
        "INTERNED_LET (6) should be 'let'"
    );
    assert_eq!(
        "change",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_CHANGE).into_tagged::<UTF8Tag>()),
        "INTERNED_CHANGE (7) should be 'change'"
    );
    assert_eq!(
        "as",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_AS).into_tagged::<UTF8Tag>()),
        "INTERNED_AS (8) should be 'as'"
    );
    assert_eq!(
        "var",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_VAR).into_tagged::<UTF8Tag>()),
        "INTERNED_VAR (9) should be 'var'"
    );
    assert_eq!(
        "nest",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_NEST).into_tagged::<UTF8Tag>()),
        "INTERNED_NEST (10) should be 'nest'"
    );
    assert_eq!(
        "complex",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_COMPLEX).into_tagged::<UTF8Tag>()),
        "INTERNED_COMPLEX (11) should be 'complex'"
    );
    assert_eq!(
        "override",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_OVERRIDE).into_tagged::<UTF8Tag>()),
        "INTERNED_OVERRIDE (12) should be 'override'"
    );
    assert_eq!(
        "true",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_TRUE).into_tagged::<UTF8Tag>()),
        "INTERNED_TRUE (13) should be 'true'"
    );
    assert_eq!(
        "false",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_FALSE).into_tagged::<UTF8Tag>()),
        "INTERNED_FALSE (14) should be 'false'"
    );
    assert_eq!(
        "IsEmpty",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_IS_EMPTY).into_tagged::<UTF8Tag>()),
        "INTERNED_IS_EMPTY (15) should be 'IsEmpty'"
    );
    assert_eq!(
        "IsWhitespace",
        interner.extract_utf8_str(
            InternedId::new(intern::INTERNED_IS_WHITESPACE).into_tagged::<UTF8Tag>()
        ),
        "INTERNED_IS_WHITESPACE (16) should be 'IsWhitespace'"
    );
    assert_eq!(
        "Range",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_RANGE).into_tagged::<UTF8Tag>()),
        "INTERNED_RANGE (17) should be 'Range'"
    );
    assert_eq!(
        "StartsW",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_STARTSW).into_tagged::<UTF8Tag>()),
        "INTERNED_STARTSW (18) should be 'StartsW'"
    );
    assert_eq!(
        "EndsW",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_ENDSW).into_tagged::<UTF8Tag>()),
        "INTERNED_ENDSW (19) should be 'EndsW'"
    );
    assert_eq!(
        "Contains",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_CONTAINS).into_tagged::<UTF8Tag>()),
        "INTERNED_CONTAINS (20) should be 'Contains'"
    );
    assert_eq!(
        "Equals",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_EQUALS).into_tagged::<UTF8Tag>()),
        "INTERNED_EQUALS (21) should be 'Equals'"
    );
}

/// Appending preserves all bytes, deduplicates existing entries, and retains UTF-8 guarantees.
#[test]
fn append_does_not_read_out_of_bounds() {
    let mut donor = Intern::init();
    donor.intern_str("shared_é");
    donor.intern_bytes(&[0xff, 0x00, 0x80]);
    donor.intern_str("");
    donor.intern_path(Path::new("shared/module.chrn"));
    donor.intern_path(Path::new("donor/module.chrn"));
    for i in 0..200 {
        donor.intern_str(&format!("user_string_{i}"));
    }

    let mut receiver = Intern::init();
    let shared = receiver.intern_bytes("shared_é".as_bytes());
    let receiver_only = receiver.intern_str("receiver_only");
    let shared_path = receiver.intern_path(Path::new("shared/module.chrn"));
    receiver.append(&donor);

    // Appending text must preserve the receiver's existing byte entry and UTF-8 guarantee.
    let shared_text = receiver
        .get_interned_id_from_str("shared_é")
        .expect("appending UTF-8 must promote the receiver's byte entry");
    assert_eq!(shared_text.inner(), shared.inner());
    assert_eq!(receiver.extract_utf8_str(shared_text), "shared_é");
    assert_eq!(receiver.extract_utf8_str(receiver_only), "receiver_only");
    let binary =
        InternedId::new(intern::INTERNER_PRELOAD_SIZE as u32 + 2).into_tagged::<MaybeUTF8Tag>();
    assert_eq!(receiver.extract_bytes(binary), &[0xff, 0x00, 0x80]);
    assert_eq!(receiver.intern_bytes(&[0xff, 0x00, 0x80]), binary);
    let empty = receiver
        .get_interned_id_from_str("")
        .expect("missing empty string");
    assert_eq!(empty.inner().id, binary.inner().id + 1);
    assert_eq!(receiver.extract_utf8_str(empty), "");
    for i in 0..200 {
        let original = format!("user_string_{i}");
        let id = receiver
            .get_interned_id_from_str(&original)
            .unwrap_or_else(|| panic!("missing {original} after append"));
        assert_eq!(id.inner().id, empty.inner().id + i + 1, "ID for {original}");
        assert_eq!(receiver.extract_utf8_str(id), original);
    }
    assert_eq!(
        receiver.intern_path(Path::new("shared/module.chrn")),
        shared_path
    );
    let donor_path = receiver.intern_path(Path::new("donor/module.chrn"));
    assert_eq!(donor_path.id, shared_path.id + 1);
    assert_eq!(
        receiver.extract_path_from_path_id(donor_path),
        Path::new("donor/module.chrn")
    );

    receiver.append(&donor);
    assert_eq!(receiver.intern_bytes(&[0xff, 0x00, 0x80]), binary);
    assert_eq!(
        receiver.intern_str("after_append").inner().id,
        empty.inner().id + 201
    );
}

#[test]
fn append_donor_at_exact_preload_size_boundary() {
    let mut donor = Intern::init();
    let mut receiver = Intern::init();
    receiver.append(&donor);
    let boundary = donor.intern_str("boundary_string");
    receiver.append(&donor);
    assert_eq!(
        receiver.get_interned_id_from_str("boundary_string"),
        Some(boundary)
    );
    assert_eq!(receiver.extract_utf8_str(boundary), "boundary_string");
    assert_eq!(
        receiver.intern_str("after_boundary").inner().id,
        boundary.inner().id + 1
    );
}

#[test]
fn append_merges_utf8_guarantees_without_demoting_or_duplicating_entries() {
    let cases = [
        ("bytes_bytes_é", false, false),
        ("bytes_text_水", false, true),
        ("text_bytes_🦀", true, false),
        ("text_text_é", true, true),
    ];
    let mut receiver = Intern::init();
    let mut donor = Intern::init();
    let mut receiver_ids = Vec::new();
    for (value, receiver_utf8, _) in cases {
        let id = receiver.intern_bytes(value.as_bytes());
        if receiver_utf8 {
            assert_eq!(receiver.intern_str(value).inner(), id.inner());
        }
        let expected = receiver_utf8.then(|| id.inner().into_tagged::<UTF8Tag>());
        assert_eq!(
            receiver.get_interned_id_from_str(value),
            expected,
            "{value} before append"
        );
        receiver_ids.push(id);
    }
    // Different insertion orders ensure donor IDs cannot stand in for receiver IDs.
    for (value, _, donor_utf8) in cases.into_iter().rev() {
        if donor_utf8 {
            donor.intern_str(value);
        } else {
            donor.intern_bytes(value.as_bytes());
        }
    }
    donor.intern_bytes(b"donor_only_raw");
    let raw_only = InternedId::new(intern::INTERNER_PRELOAD_SIZE as u32 + cases.len() as u32)
        .into_tagged::<MaybeUTF8Tag>();

    for round in 0..2 {
        receiver.append(&donor);
        for ((value, receiver_utf8, donor_utf8), id) in cases.into_iter().zip(&receiver_ids) {
            let expected =
                (receiver_utf8 || donor_utf8).then(|| id.inner().into_tagged::<UTF8Tag>());
            assert_eq!(
                receiver.get_interned_id_from_str(value),
                expected,
                "{value}, append {round}"
            );
            assert_eq!(
                receiver.get_bytes(*id),
                Some(value.as_bytes()),
                "{value}, append {round}"
            );
            assert_eq!(
                receiver.intern_bytes(value.as_bytes()),
                *id,
                "{value}, append {round}"
            );
            // Byte access and deduplication must not erase an existing UTF-8 guarantee.
            assert_eq!(
                receiver.get_interned_id_from_str(value),
                expected,
                "{value} after byte access"
            );
        }
        assert_eq!(receiver.get_bytes(raw_only), Some(&b"donor_only_raw"[..]));
        assert_eq!(receiver.get_interned_id_from_str("donor_only_raw"), None);
    }
    assert_eq!(
        receiver.intern_str("after_merge").inner().id,
        raw_only.inner().id + 1,
        "repeated append and byte deduplication must not allocate extra entries"
    );
}

#[test]
fn tagged_strings_and_bytes_survive_growth_and_deduplicate() {
    let mut interner = Intern::init();
    let empty_bytes = interner.intern_bytes(b"");
    let empty_text = interner.intern_str("");
    assert_eq!(empty_bytes.inner(), empty_text.inner());
    let text = interner.intern_str("é\0水");
    let text_bytes = interner.intern_bytes("é\0水".as_bytes());
    assert_eq!(text.inner(), text_bytes.inner());
    assert_eq!(interner.get_interned_id_from_str("é\0水"), Some(text));
    let bytes_first = interner.intern_bytes("bytes_first_🦀".as_bytes());
    for _ in 0..2 {
        assert_eq!(
            interner.get_interned_id_from_str("bytes_first_🦀"),
            None,
            "lookup must not promote byte-first entries"
        );
    }
    let promoted = interner.intern_str("bytes_first_🦀");
    assert_eq!(bytes_first.inner(), promoted.inner());
    let looked_up = interner
        .get_interned_id_from_str("bytes_first_🦀")
        .expect("explicitly promoted UTF-8 text must be found");
    assert_eq!(looked_up, promoted);
    assert_eq!(interner.extract_utf8_str(looked_up), "bytes_first_🦀");
    let binary = interner.intern_bytes(&[0xff, 0xc0, 0x00, 0x80]);
    assert_eq!(binary.inner().id, bytes_first.inner().id + 1);
    assert_eq!(interner.intern_bytes(&[0xff, 0xc0, 0x00, 0x80]), binary);

    let mut added = Vec::new();
    for i in 0..512 {
        let value = format!("growth_{i}_{}", "水".repeat(i % 37));
        added.push((interner.intern_str(&value), value));
    }
    assert_eq!(interner.extract_utf8_str(empty_text), "");
    assert_eq!(interner.extract_bytes(empty_bytes), b"");
    assert_eq!(interner.extract_utf8_str(text), "é\0水");
    assert_eq!(interner.extract_bytes(text_bytes), "é\0水".as_bytes());
    assert_eq!(interner.extract_utf8_str(promoted), "bytes_first_🦀");
    assert_eq!(interner.extract_bytes(binary), &[0xff, 0xc0, 0x00, 0x80]);
    for (id, value) in added {
        assert_eq!(interner.extract_utf8_str(id), value);
        assert_eq!(interner.intern_str(&value), id);
    }
}

#[test]
fn checked_access_handles_missing_ids_and_promoted_byte_first_text() {
    let get_utf8_str: fn(&Intern, InternedIdUTF8) -> Option<&str> = Intern::get_utf8_str;
    let mut interner = Intern::init();
    let missing =
        InternedId::new(intern::INTERNER_PRELOAD_SIZE as u32).into_tagged::<MaybeUTF8Tag>();
    let missing_text =
        InternedId::new(intern::INTERNER_PRELOAD_SIZE as u32).into_tagged::<UTF8Tag>();
    assert_eq!(interner.get_bytes(missing), None);
    assert_eq!(get_utf8_str(&interner, missing_text), None);
    assert_eq!(interner.get_interned_id_from_str("missing"), None);
    assert_eq!(interner.get_path_from_path_id(PathId::new(0)), None);

    for value in ["", "é\0水"] {
        let id = interner.intern_bytes(value.as_bytes());
        assert_eq!(interner.get_bytes(id), Some(value.as_bytes()));
        assert_eq!(interner.extract_bytes(id), value.as_bytes());
        // The caller knows these bytes came from a str; reading must trust that tag.
        let asserted_text_id = id.inner().into_tagged::<UTF8Tag>();
        assert_eq!(get_utf8_str(&interner, asserted_text_id), Some(value));
        assert_eq!(interner.get_interned_id_from_str(value), None);
        let text_id = interner.intern_str(value);
        assert_eq!(text_id.inner(), id.inner());
        assert_eq!(get_utf8_str(&interner, text_id), Some(value));
        assert_eq!(interner.get_bytes(text_id), Some(value.as_bytes()));
        assert_eq!(interner.extract_bytes(text_id), value.as_bytes());
        assert_eq!(interner.get_interned_id_from_str(value), Some(text_id));
        assert_eq!(interner.extract_utf8_str(text_id), value);
    }
    let binary = interner.intern_bytes(&[0xff, 0x00, 0x80]);
    assert_eq!(interner.get_bytes(binary), Some(&[0xff, 0x00, 0x80][..]));
    assert_eq!(interner.extract_bytes(binary), &[0xff, 0x00, 0x80]);
    let after_binary = InternedId::new(binary.inner().id + 1).into_tagged::<MaybeUTF8Tag>();
    let after_binary_text = InternedId::new(binary.inner().id + 1).into_tagged::<UTF8Tag>();
    assert_eq!(interner.get_bytes(after_binary), None);
    assert_eq!(get_utf8_str(&interner, after_binary_text), None);
    assert_eq!(interner.get_interned_id_from_str("missing"), None);
    assert_eq!(interner.intern_bytes(b"missing"), after_binary);

    let path = Path::new("checked/module.chrn");
    let path_id = interner.intern_path(path);
    assert_eq!(path_id, PathId::new(0));
    assert_eq!(interner.get_path_from_path_id(path_id), Some(path));
    assert_eq!(interner.extract_path_from_path_id(path_id), path);
    assert_eq!(interner.get_path_from_path_id(PathId::new(1)), None);
    assert_eq!(interner.intern_path(path), path_id);
}

#[test]
fn builtin_type_intern_alignment() {
    let interner = Intern::init();

    assert_eq!(
        "i8",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I8).into_tagged::<UTF8Tag>()),
        "INTERNED_I8 (22) should be 'i8'"
    );
    assert_eq!(
        "u8",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U8).into_tagged::<UTF8Tag>()),
        "INTERNED_U8 (23) should be 'u8'"
    );
    assert_eq!(
        "i16",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I16).into_tagged::<UTF8Tag>()),
        "INTERNED_I16 (24) should be 'i16'"
    );
    assert_eq!(
        "u16",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U16).into_tagged::<UTF8Tag>()),
        "INTERNED_U16 (25) should be 'u16'"
    );
    assert_eq!(
        "i32",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I32).into_tagged::<UTF8Tag>()),
        "INTERNED_I32 (26) should be 'i32'"
    );
    assert_eq!(
        "u32",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U32).into_tagged::<UTF8Tag>()),
        "INTERNED_U32 (27) should be 'u32'"
    );
    assert_eq!(
        "f32",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_F32).into_tagged::<UTF8Tag>()),
        "INTERNED_F32 (28) should be 'f32'"
    );
    assert_eq!(
        "i64",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I64).into_tagged::<UTF8Tag>()),
        "INTERNED_I64 (29) should be 'i64'"
    );
    assert_eq!(
        "u64",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U64).into_tagged::<UTF8Tag>()),
        "INTERNED_U64 (30) should be 'u64'"
    );
    assert_eq!(
        "f64",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_F64).into_tagged::<UTF8Tag>()),
        "INTERNED_F64 (31) should be 'f64'"
    );
    assert_eq!(
        "i128",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I128).into_tagged::<UTF8Tag>()),
        "INTERNED_I128 (32) should be 'i128'"
    );
    assert_eq!(
        "u128",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U128).into_tagged::<UTF8Tag>()),
        "INTERNED_U128 (33) should be 'u128'"
    );
    assert_eq!(
        "f128",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_F128).into_tagged::<UTF8Tag>()),
        "INTERNED_F128 (34) should be 'f128'"
    );
    assert_eq!(
        "sized",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_SIZED).into_tagged::<UTF8Tag>()),
        "INTERNED_SIZED (35) should be 'sized'"
    );
    assert_eq!(
        "unsized",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_UNSIZED).into_tagged::<UTF8Tag>()),
        "INTERNED_UNSIZED (36) should be 'unsized'"
    );
    assert_eq!(
        "bool",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_BOOL).into_tagged::<UTF8Tag>()),
        "INTERNED_BOOL (37) should be 'bool'"
    );
    assert_eq!(
        "nil",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_NIL).into_tagged::<UTF8Tag>()),
        "INTERNED_NIL (38) should be 'nil'"
    );
    assert_eq!(
        "char",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_CHAR).into_tagged::<UTF8Tag>()),
        "INTERNED_CHAR (39) should be 'char'"
    );
    assert_eq!(
        "str",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_STR).into_tagged::<UTF8Tag>()),
        "INTERNED_STR (40) should be 'str'"
    );
    assert_eq!(
        "BigInt",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_BIGINT).into_tagged::<UTF8Tag>()),
        "INTERNED_BIGINT (41) should be 'BigInt'"
    );
    assert_eq!(
        "BigFloat",
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_BIGFLOAT).into_tagged::<UTF8Tag>()),
        "INTERNED_BIGFLOAT (42) should be 'BigFloat'"
    );
    assert_eq!(
        "List",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_LIST).into_tagged::<UTF8Tag>()),
        "INTERNED_LIST (43) should be 'List'"
    );
    assert_eq!(
        "Set",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_SET).into_tagged::<UTF8Tag>()),
        "INTERNED_SET (44) should be 'Set'"
    );
    assert_eq!(
        "Map",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_MAP).into_tagged::<UTF8Tag>()),
        "INTERNED_MAP (45) should be 'Map'"
    );
    assert_eq!(
        "Tuple",
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_TUPLE).into_tagged::<UTF8Tag>()),
        "INTERNED_TUPLE (46) should be 'Tuple'"
    );
}

#[test]
fn builtin_type_kind_alignment() {
    let interner = Intern::init();

    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I8).into_tagged::<UTF8Tag>()),
        "i8"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U8).into_tagged::<UTF8Tag>()),
        "u8"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I16).into_tagged::<UTF8Tag>()),
        "i16"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U16).into_tagged::<UTF8Tag>()),
        "u16"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I32).into_tagged::<UTF8Tag>()),
        "i32"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U32).into_tagged::<UTF8Tag>()),
        "u32"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_F32).into_tagged::<UTF8Tag>()),
        "f32"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I64).into_tagged::<UTF8Tag>()),
        "i64"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U64).into_tagged::<UTF8Tag>()),
        "u64"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_F64).into_tagged::<UTF8Tag>()),
        "f64"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_I128).into_tagged::<UTF8Tag>()),
        "i128"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_U128).into_tagged::<UTF8Tag>()),
        "u128"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_F128).into_tagged::<UTF8Tag>()),
        "f128"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_SIZED).into_tagged::<UTF8Tag>()),
        "sized"
    );
    assert_eq!(
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_UNSIZED).into_tagged::<UTF8Tag>()),
        "unsized"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_BOOL).into_tagged::<UTF8Tag>()),
        "bool"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_NIL).into_tagged::<UTF8Tag>()),
        "nil"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_CHAR).into_tagged::<UTF8Tag>()),
        "char"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_STR).into_tagged::<UTF8Tag>()),
        "str"
    );
    assert_eq!(
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_BIGINT).into_tagged::<UTF8Tag>()),
        "BigInt"
    );
    assert_eq!(
        interner
            .extract_utf8_str(InternedId::new(intern::INTERNED_BIGFLOAT).into_tagged::<UTF8Tag>()),
        "BigFloat"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_LIST).into_tagged::<UTF8Tag>()),
        "List"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_SET).into_tagged::<UTF8Tag>()),
        "Set"
    );
    assert_eq!(
        interner.extract_utf8_str(InternedId::new(intern::INTERNED_MAP).into_tagged::<UTF8Tag>()),
        "Map"
    );
}

#[test]
fn interned_preload_size_matches() {
    let mut interner = Intern::init();
    for (text, expected_id) in intern::PRELOADED_STRINGS {
        assert_eq!(
            interner.get_interned_id_from_str(text),
            Some(InternedId::new(expected_id).into_tagged::<UTF8Tag>()),
            "preload {text} must already have a UTF-8 guarantee"
        );
        let id = interner.intern_str(text);
        assert_eq!(id.inner().id, expected_id, "preload {text}");
        assert_eq!(interner.extract_utf8_str(id), text, "preload {expected_id}");
    }
    assert_eq!(
        interner.intern_str("first_user_string").inner().id as usize,
        intern::INTERNER_PRELOAD_SIZE,
        "first user string must follow every preloaded entry"
    );
}
