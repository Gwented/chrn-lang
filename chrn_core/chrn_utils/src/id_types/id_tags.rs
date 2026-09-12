use std::marker::PhantomData;

use crate::id_types::ArenaIndex;
// The trait could just be defined in utils as well as the struct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaggedId<I: ArenaIndex, T: ArenaIndexTag> {
    inner: I,
    _phantom_data: PhantomData<T>,
}

// Uses getters and setters for the sake of easier searching if any bug were to occur
impl<I: ArenaIndex, T: ArenaIndexTag> TaggedId<I, T> {
    pub const fn new(inner: I) -> Self {
        Self {
            inner,
            _phantom_data: PhantomData,
        }
    }

    /// Returns `self.inner`
    pub const fn inner(&self) -> I {
        self.inner
    }

    /// Sets `self.inner` to `val`
    pub const fn set_inner(&mut self, val: I) {
        self.inner = val;
    }
}

pub trait ArenaIndexTag: Copy {}

/// Helper macro for tag declarations
#[macro_export]
macro_rules! tag_decl {
    ($($ident:ident),* $(,)?) => {
        $(
        /// Tag for restricting `ArenaIndex`'s that have broad applications.
        #[repr(transparent)]
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
        pub struct $ident;
        impl ArenaIndexTag for $ident {}
        )*
    }
}

// Is this ok?
/// Clones a given `Tagged<T,I>` into a `Vec` with only `I`
pub fn clone_vec_untagged<I: ArenaIndex, T: ArenaIndexTag>(tagged: &[TaggedId<I, T>]) -> Vec<I> {
    tagged.iter().map(|tag| tag.inner).collect()
}
