use std::ops::{Deref, DerefMut};

/// Simple arena of Strings. Strings are either dead, meaning they contain no
/// useful data, or live, meaning that they do contain valid data. To get
/// strings, use the `take_live` or `take_dead` functions as appropriate.
/// Strings will automatically be returned to the opposite pool when they are
/// dropped.
pub struct StringPool {
    live: Vec<String>,
    dead: Vec<String>,
}

enum Target {
    Live,
    Dead,
}

/// Reference to a string from a LinePool. Will automatically be returned to the
/// appropriate collection (live/dead) when Dropped. Implements `Deref` and
/// `DerefMut` with `String` as the target.
pub struct PoolRef<'a> {
    line: String,
    pool: &'a mut StringPool,
    target: Target,
}

impl StringPool {
    /// Generate a new string pool containing `capacity` dead strings with
    /// capacity for the same number of live strings.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut dead = Vec::with_capacity(capacity);
        dead.resize(capacity, String::new());

        Self {
            dead,
            live: Vec::with_capacity(capacity),
        }
    }

    /// Is the pool of live strings empty
    pub fn is_empty(&self) -> bool {
        self.live.is_empty()
    }

    /// Length of the pool of live strings
    pub fn len(&self) -> usize {
        self.live.len()
    }

    /// Pull a live string from the pool so that it can be used. This string
    /// must have had a valid value placed into it during a previous call to
    /// `take_dead`. Users of this function may assume that the string contains
    /// valid data.
    pub fn take_live(&mut self) -> Option<PoolRef> {
        let line = self.live.pop()?;

        Some(PoolRef {
            line,
            pool: self,
            target: Target::Dead,
        })
    }

    /// Pull a dead string from the pool so that it can have a valid value put
    /// into it. Returned string will be cleared of previous data, but will
    /// retain the same buffer and will therefore reuse allocations.
    pub fn take_dead(&mut self) -> Option<PoolRef> {
        let mut line = self.dead.pop()?;

        line.clear();

        Some(PoolRef {
            line,
            pool: self,
            target: Target::Live,
        })
    }

    /// Maximum number of items that can be stored in this pool
    pub fn capacity(&self) -> usize {
        self.live.capacity()
    }
}

impl<'a> PoolRef<'a> {
    /// Ensure that this ref will be returned to the dead pool when it is
    /// dropped. This is useful on a ref returned from `take_dead` if the
    /// operation to fill it with valid data fails for some reason.
    pub fn stay_dead(&mut self) {
        self.target = Target::Dead;
    }
}

impl<'a> Drop for PoolRef<'a> {
    fn drop(&mut self) {
        let vec = match self.target {
            Target::Dead => &mut self.pool.dead,
            Target::Live => &mut self.pool.live,
        };

        vec.push(std::mem::take(&mut self.line));
    }
}

impl<'a> Deref for PoolRef<'a> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.line
    }
}

impl<'a> DerefMut for PoolRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.line
    }
}
