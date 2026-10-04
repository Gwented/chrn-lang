/// Wrapper which contains operations useful for traversing a slice of elements
#[derive(Debug, Clone)]
pub struct BasicCursor<'a, T> {
    pub items: &'a [T],
    pos: usize,
}

impl<'a, T> BasicCursor<'a, T> {
    pub const fn new(items: &'a [T]) -> Self {
        Self { pos: 0, items }
    }

    /// Sets initial position
    pub const fn with_pos(items: &'a [T], pos: usize) -> Self {
        debug_assert!(pos <= items.len(), "`with_pos` misusage");
        Self { items, pos }
    }

    /// Clones the current item and advances by one. Returns `None` when pos >= len.
    pub fn advance_owned_checked(&mut self) -> Option<T>
    where
        T: Clone,
    {
        if self.pos >= self.len() {
            return None;
        }
        Some(self.advance_owned())
    }

    /// Clones the current item then advances by one. Panics when pos >= len.
    pub fn advance_owned(&mut self) -> T
    where
        T: Clone,
    {
        let current = self.peek_owned();
        self.pos += 1;
        current
    }

    /// Borrows the current item then advances by one. Returns `None` when pos >= len.
    pub const fn advance_ref_checked(&mut self) -> Option<&'a T> {
        if self.pos >= self.len() {
            return None;
        }
        Some(self.advance_ref())
    }

    /// Borrows the current item then advances by one. Panics when pos >= len.
    pub const fn advance_ref(&mut self) -> &'a T {
        let current = self.peek_ref();
        self.pos += 1;
        current
    }

    pub fn peek_ahead_owned_checked(&self, amt: usize) -> Option<T>
    where
        T: Clone,
    {
        self.peek_ahead_ref_checked(amt).cloned()
    }

    pub fn peek_ahead_owned(&self, amt: usize) -> T
    where
        T: Clone,
    {
        self.peek_ahead_ref(amt).clone()
    }

    pub const fn peek_ahead_ref_checked(&self, amt: usize) -> Option<&'a T> {
        match self.pos.checked_add(amt) {
            Some(sum) if sum < self.len() => Some(&self.items[sum]),
            _ => None,
        }
    }

    pub const fn peek_ahead_ref(&self, amt: usize) -> &'a T {
        let idx = self.pos.checked_add(amt).expect("cursor position overflow");
        &self.items[idx]
    }

    pub fn peek_behind_owned_checked(&self, amt: usize) -> Option<T>
    where
        T: Clone,
    {
        self.peek_behind_ref_checked(amt).cloned()
    }

    pub fn peek_behind_owned(&self, amt: usize) -> T
    where
        T: Clone,
    {
        self.peek_behind_ref(amt).clone()
    }

    pub const fn peek_behind_ref_checked(&self, amt: usize) -> Option<&'a T> {
        match self.pos.checked_sub(amt) {
            Some(diff) if diff < self.len() => Some(&self.items[diff]),
            _ => None,
        }
    }

    pub const fn peek_behind_ref(&self, amt: usize) -> &'a T {
        let idx = self
            .pos
            .checked_sub(amt)
            .expect("cursor position underflow");
        &self.items[idx]
    }

    /// Borrows item at `self.pos`. Returns `None` when pos >= len.
    pub const fn peek_ref_checked(&self) -> Option<&'a T> {
        self.peek_ahead_ref_checked(0)
    }

    /// Borrows item at `self.pos`. Panics when pos >= len.
    pub const fn peek_ref(&self) -> &'a T {
        &self.items[self.pos]
    }

    /// Clones item at `self.pos`. Returns `None` when pos >= len.
    pub fn peek_owned_checked(&self) -> Option<T>
    where
        T: Clone,
    {
        self.peek_ref_checked().cloned()
    }

    /// Peeks
    pub fn peek_owned(&self) -> T
    where
        T: Clone,
    {
        self.peek_ref().clone()
    }

    /// Advances without checking if pos + amt exceeds the len.
    /// Panics if the position overflows.
    pub const fn skip(&mut self, amt: usize) {
        self.pos = self.pos.checked_add(amt).expect("cursor position overflow");
    }

    /// Advances by amt. Returns false without moving if the destination exceeds len or overflows.
    pub const fn skip_checked(&mut self, amt: usize) -> bool {
        match self.pos.checked_add(amt) {
            Some(sum) if sum <= self.len() => {
                self.pos = sum;
                true
            }
            _ => false,
        }
    }

    pub const fn pos(&self) -> usize {
        self.pos
    }

    pub const fn len(&self) -> usize {
        self.items.len()
    }
}
