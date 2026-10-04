use crate::cursors::BasicCursor;

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "`with_pos` misusage")]
fn cursor_with_pos_rejects_positions_beyond_slice_in_debug_builds() {
    BasicCursor::with_pos(&[10], 2);
}

#[test]
fn cursor_checked_reads_validate_destination_without_moving() {
    let items = [10, 20, 30];
    let cursor = BasicCursor::new(&items);
    assert_eq!(cursor.peek_behind_ref_checked(1), None);
    assert_eq!(cursor.peek_behind_owned_checked(usize::MAX), None);
    assert_eq!(cursor.pos(), 0);

    let cursor = BasicCursor::with_pos(&items, 1);
    assert_eq!(cursor.peek_ref_checked(), Some(&20));
    assert_eq!(cursor.peek_ahead_ref_checked(0), Some(&20));
    assert_eq!(cursor.peek_behind_owned_checked(0), Some(20));
    assert_eq!(cursor.peek_ahead_owned_checked(1), Some(30));
    assert_eq!(cursor.peek_behind_ref_checked(1), Some(&10));
    assert_eq!(cursor.peek_ahead_ref_checked(2), None);
    assert_eq!(cursor.peek_ahead_owned_checked(usize::MAX), None);
    assert_eq!(cursor.pos(), 1);

    let mut cursor = BasicCursor::with_pos(&items, items.len());
    assert_eq!(cursor.peek_ref_checked(), None);
    assert_eq!(cursor.peek_owned_checked(), None);
    assert_eq!(cursor.peek_behind_ref_checked(0), None);
    assert_eq!(cursor.peek_behind_owned_checked(1), Some(30));
    assert_eq!(cursor.advance_ref_checked(), None);
    assert_eq!(cursor.advance_owned_checked(), None);
    assert_eq!(cursor.pos(), items.len());
}

#[test]
fn cursor_checked_skip_accepts_eof_and_preserves_position_on_failure() {
    let mut cursor = BasicCursor::new(&[10, 20]);
    assert!(cursor.skip_checked(1));
    assert_eq!(cursor.pos(), 1);
    assert!(!cursor.skip_checked(usize::MAX));
    assert!(!cursor.skip_checked(2));
    assert_eq!(cursor.pos(), 1);
    assert!(cursor.skip_checked(1));
    assert_eq!(cursor.pos(), 2);
    assert!(cursor.skip_checked(0));
    assert!(!cursor.skip_checked(1));
    assert_eq!(cursor.pos(), 2);

    let mut empty = BasicCursor::<u8>::new(&[]);
    assert_eq!(empty.len(), 0);
    assert!(empty.skip_checked(0));
    assert!(!empty.skip_checked(1));
    assert_eq!(empty.peek_ref_checked(), None);
    assert_eq!(empty.peek_owned_checked(), None);
    assert_eq!(empty.peek_ahead_ref_checked(0), None);
    assert_eq!(empty.peek_ahead_owned_checked(1), None);
    assert_eq!(empty.advance_ref_checked(), None);
    assert_eq!(empty.advance_owned_checked(), None);
    assert_eq!(empty.peek_behind_ref_checked(0), None);
    assert_eq!(empty.peek_behind_owned_checked(1), None);
    assert_eq!(empty.pos(), 0);
}

#[test]
fn cursor_checked_operations_support_positions_beyond_replaced_slice() {
    let items = [10, 20, 30];
    let mut cursor = BasicCursor::new(&items);
    cursor.skip(4);
    assert_eq!(cursor.peek_behind_ref_checked(2), Some(&30));
    assert_eq!(cursor.peek_behind_ref_checked(0), None);
    assert_eq!(cursor.advance_ref_checked(), None);
    assert!(!cursor.skip_checked(0));
    assert_eq!(cursor.pos(), 4);

    cursor.items = &items[..1];
    assert_eq!(cursor.peek_behind_owned_checked(4), Some(10));
    assert_eq!(cursor.peek_behind_ref_checked(2), None);
    assert_eq!(cursor.peek_owned_checked(), None);
    assert_eq!(cursor.advance_owned_checked(), None);
    assert!(!cursor.skip_checked(1));
    assert_eq!(cursor.pos(), 4);

    cursor.items = &items;
    assert_eq!(cursor.len(), 3);
    cursor.skip(usize::MAX - cursor.pos());
    assert_eq!(cursor.pos(), usize::MAX);
    assert_eq!(cursor.peek_ahead_ref_checked(1), None);
    assert_eq!(cursor.peek_ahead_owned_checked(1), None);
    assert_eq!(cursor.peek_behind_ref_checked(usize::MAX), Some(&10));
    assert!(!cursor.skip_checked(1));
    assert_eq!(cursor.pos(), usize::MAX);
}

#[test]
fn cursor_unchecked_peeks_use_offsets_without_moving() {
    let items = [
        String::from("first"),
        String::from("middle"),
        String::from("last"),
    ];
    let cursor = BasicCursor::with_pos(&items, 1);
    assert_eq!(cursor.len(), 3);
    assert!(std::ptr::eq(cursor.peek_ref(), &items[1]));
    assert!(std::ptr::eq(cursor.peek_ahead_ref(1), &items[2]));
    assert!(std::ptr::eq(cursor.peek_behind_ref(1), &items[0]));
    assert_eq!(cursor.peek_ahead_ref(0), &items[1]);
    assert_eq!(cursor.peek_behind_ref(0), &items[1]);

    for mut owned in [
        cursor.peek_owned(),
        cursor.peek_ahead_owned(0),
        cursor.peek_behind_owned(0),
    ] {
        assert_eq!(owned, "middle");
        owned.push('!');
    }
    assert_eq!(cursor.peek_ahead_owned(1), "last");
    assert_eq!(cursor.peek_behind_owned(1), "first");
    assert_eq!(items[1], "middle");
    assert_eq!(cursor.pos(), 1);
}

#[test]
fn cursor_clones_share_items_but_advance_independently() {
    let items = [10, 20, 30];
    let mut cursor = BasicCursor::with_pos(&items, 1);
    let mut cloned = cursor.clone();
    assert!(std::ptr::eq(cursor.items, cloned.items));
    assert_eq!(cloned.pos(), 1);
    cursor.skip(0);
    assert_eq!(cursor.pos(), 1);
    cursor.skip(1);
    assert_eq!(cursor.peek_ref(), &30);
    assert_eq!(cloned.advance_owned_checked(), Some(20));
    assert_eq!(cloned.pos(), 2);
    assert_eq!(cloned.advance_ref_checked(), Some(&30));
    assert_eq!(cloned.pos(), 3);
    assert_eq!(cursor.pos(), 2);
}

#[test]
fn cursor_borrowed_items_survive_advancement_without_clone() {
    #[derive(Debug, PartialEq)]
    struct Item(u8);

    let items = [Item(10), Item(20)];
    let mut cursor = BasicCursor::new(&items);
    let first = cursor.advance_ref_checked().unwrap();
    let second = cursor.advance_ref();
    assert_eq!(first, &Item(10));
    assert_eq!(second, &Item(20));
    assert!(std::ptr::eq(first, &items[0]));
    assert!(std::ptr::eq(second, &items[1]));
    assert_eq!(cursor.pos(), 2);

    let items = [String::from("first"), String::from("second")];
    let mut cursor = BasicCursor::new(&items);
    let mut cloned = cursor.peek_owned_checked().unwrap();
    assert_eq!(cloned, "first");
    cloned.push('!');
    assert_eq!(cursor.advance_owned_checked(), Some(String::from("first")));
    assert_eq!(cursor.advance_owned(), "second");
    assert_eq!(items[0], "first");
    assert_eq!(cursor.pos(), 2);
}

#[test]
fn cursor_panicking_operations_reject_arithmetic_wraparound() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    let items = [10, 20];
    let mut cursor = BasicCursor::with_pos(&items, 1);
    assert!(catch_unwind(|| cursor.peek_ahead_ref(usize::MAX)).is_err());
    assert!(catch_unwind(|| cursor.peek_ahead_owned(usize::MAX)).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| cursor.skip(usize::MAX))).is_err());
    assert_eq!(cursor.pos(), 1);

    let cursor = BasicCursor::new(&items);
    assert!(catch_unwind(|| cursor.peek_behind_ref(usize::MAX)).is_err());
    assert!(catch_unwind(|| cursor.peek_behind_owned(usize::MAX)).is_err());
}

#[test]
fn cursor_unchecked_reads_panic_at_eof_without_advancing() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    let operations: [(&str, fn(&mut BasicCursor<'_, u8>)); 8] = [
        ("peek_ref", |cursor| {
            cursor.peek_ref();
        }),
        ("peek_owned", |cursor| {
            cursor.peek_owned();
        }),
        ("peek_ahead_ref", |cursor| {
            cursor.peek_ahead_ref(0);
        }),
        ("peek_ahead_owned", |cursor| {
            cursor.peek_ahead_owned(0);
        }),
        ("peek_behind_ref", |cursor| {
            cursor.peek_behind_ref(0);
        }),
        ("peek_behind_owned", |cursor| {
            cursor.peek_behind_owned(0);
        }),
        ("advance_ref", |cursor| {
            cursor.advance_ref();
        }),
        ("advance_owned", |cursor| {
            cursor.advance_owned();
        }),
    ];
    for items in [&[][..], &[10, 20][..]] {
        for (name, operation) in operations {
            let mut cursor = BasicCursor::with_pos(items, items.len());
            assert!(
                catch_unwind(AssertUnwindSafe(|| operation(&mut cursor))).is_err(),
                "{name}"
            );
            assert_eq!(cursor.pos(), items.len(), "{name}");
        }
    }

    let mut cursor = BasicCursor::with_pos(&[10, 20], 1);
    assert!(catch_unwind(|| cursor.peek_ahead_ref(1)).is_err());
    assert!(catch_unwind(|| cursor.peek_ahead_owned(1)).is_err());
    assert!(catch_unwind(|| cursor.peek_behind_ref(2)).is_err());
    assert!(catch_unwind(|| cursor.peek_behind_owned(2)).is_err());
    assert_eq!(cursor.pos(), 1);
    cursor.skip(usize::MAX - cursor.pos());
    assert!(catch_unwind(AssertUnwindSafe(|| cursor.advance_ref())).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| cursor.advance_owned())).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| cursor.skip(1))).is_err());
    assert_eq!(cursor.pos(), usize::MAX);
}
