#[derive(Debug, Clone)]
pub struct CursorBasic<'a, T> {
    pos: usize,
    pub items: &'a [T],
}

impl<'a, T> CursorBasic<'a, T> {
    pub const fn new(items: &'a [T]) -> Self {
        Self { pos: 0, items }
    }

    pub const fn with_pos(items: &'a [T], pos: usize) -> Self {
        Self { items, pos }
    }

    /// Returns `None` if advancing would exceed the len
    pub fn advance_owned_checked(&mut self) -> Option<T>
    where
        T: Clone,
    {
        if self.pos >= self.items.len() {
            return None;
        }
        Some(self.advance_owned())
    }

    /// Returns current pos `T` then increments len + 1 unchecked
    pub fn advance_owned(&mut self) -> T
    where
        T: Clone,
    {
        let current = self.items[self.pos].clone();
        self.increment_pos();
        current
    }

    /// Returns `None` if advancing would exceed the len
    pub const fn advance_ref_checked(&mut self) -> Option<&T>
    where
        T: Clone,
    {
        if self.pos >= self.items.len() {
            return None;
        }
        Some(self.advance_ref())
    }

    /// Returns current pos `&T` then increments len + 1 unchecked
    pub const fn advance_ref(&mut self) -> &T {
        let current = &self.items[self.pos];
        self.pos += 1;
        current
    }

    pub fn peek_ahead_owned_checked(&self, amt: usize) -> Option<T>
    where
        T: Clone,
    {
        if self.is_over_len(amt) {
            return None;
        }
        Some(self.items[self.pos + amt].clone())
    }

    pub fn peek_ahead_owned(&self, amt: usize) -> T
    where
        T: Clone,
    {
        self.items[self.pos + amt].clone()
    }

    pub const fn peek_ahead_ref_checked(&self, amt: usize) -> Option<&T> {
        if self.is_over_len(amt) {
            return None;
        }
        Some(&self.items[self.pos + amt])
    }

    pub const fn peek_ahead_ref(&self, amt: usize) -> &T {
        &self.items[self.pos + amt]
    }

    pub fn peek_behind_owned_checked(&self, amt: usize) -> Option<T>
    where
        T: Clone,
    {
        if self.is_under_len(amt) {
            return None;
        }
        Some(self.items[self.pos - amt].clone())
    }

    pub fn peek_behind_owned(&self, amt: usize) -> T
    where
        T: Clone,
    {
        self.items[self.pos - amt].clone()
    }

    pub const fn peek_behind_ref_checked(&self, amt: usize) -> Option<&T> {
        if self.is_under_len(amt) {
            return None;
        }
        Some(&self.items[self.pos - amt])
    }

    pub const fn peek_behind_ref(&self, amt: usize) -> &T {
        &self.items[self.pos - amt]
    }

    /// Peeks current pos and returns `&T`
    pub const fn peek_ref(&self) -> &T {
        &self.items[self.pos]
    }

    /// Peeks current pos and returns cloned `T`
    pub fn peek_owned(&self) -> T
    where
        T: Clone,
    {
        self.items[self.pos].clone()
    }

    /// Advances `self.pos` + `amt` unchecked
    pub const fn skip(&mut self, amt: usize) {
        self.increment_pos_many(amt);
    }

    /// Returns `true` if a skip was possible, `false` if skipping would exceed the len
    pub const fn skip_checked(&mut self, amt: usize) -> bool {
        if self.is_over_len(amt) {
            return false;
        }
        self.pos = self.pos + amt;
        true
    }

    const fn increment_pos(&mut self) {
        self.pos += 1;
    }

    const fn increment_pos_many(&mut self, amt: usize) {
        self.pos += amt;
    }

    const fn decrement_pos(&mut self) {
        self.pos += 1;
    }

    const fn decrement_pos_many(&mut self, amt: usize) {
        self.pos += amt;
    }

    /// Returns `true` if `self.pos` + `amt` would exceed `self.len`
    const fn is_over_len(&self, amt: usize) -> bool {
        self.pos + amt >= self.len()
    }

    /// Returns `true` if `self.pos` - `amt` would be under `self.len`
    const fn is_under_len(&self, amt: usize) -> bool {
        self.items.len().checked_sub(amt).is_none()
    }

    pub const fn pos(&self) -> usize {
        self.pos
    }

    pub const fn len(&self) -> usize {
        self.items.len()
    }
}
