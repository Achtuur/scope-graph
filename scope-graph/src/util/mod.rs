mod display;

pub use display::*;

use std::{hash::Hash, hint::black_box, mem::MaybeUninit};

use rand::{rngs::SmallRng, Rng, SeedableRng};
pub enum ContainsContainer<'a, T: Eq + Hash, const N: usize> {
    Array {
        arr: [MaybeUninit<&'a T>; N],
        ptr: usize,
    },
    StdSet(std::collections::HashSet<&'a T>),
    BrownSet(hashbrown::HashSet<&'a T>),
}

impl<'a, T: Eq + Hash, const N: usize> ContainsContainer<'a, T, N> {
    pub fn new() -> Self {
        Self::Array {
            arr: [MaybeUninit::uninit(); N],
            ptr: 0
        }
    }

    /// Inserts item and return true if collection already contained it
    pub fn insert(&mut self, item: &'a T) -> bool {
        match self {
            Self::Array { arr, ptr } => {
                if *ptr >= N {
                    self.upgrade();
                    self.insert(item)
                } else {
                    let contains = arr.iter()
                        .take(*ptr)
                        // safety: ptr keeps track of length
                        .map(|i| unsafe { i.assume_init_ref() })
                        .any(|i| *i == item);
                    arr[*ptr] = MaybeUninit::new(item);
                    *ptr += 1;
                    contains
                }
            },
            Self::StdSet(set) => set.insert(item),
            Self::BrownSet(set) => set.insert(item),
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        match self {
            Self::Array { arr, ptr } => {
                arr.iter()
                    .take(*ptr)
                    // safety: ptr keeps track of length
                    .map(|i| unsafe { i.assume_init_ref() })
                    .any(|i| *i == item)
            },
            Self::StdSet(set) => set.contains(item),
            Self::BrownSet(set) => set.contains(item),
        }
    }

    pub fn clear(&mut self) {
        match self {
            Self::Array { ptr, .. } => *ptr = 0,
            Self::StdSet(set) => set.clear(),
            Self::BrownSet(set) => set.clear(),
        }
    }

    pub fn upgrade(&mut self) {
        match self {
            Self::Array { arr, ptr } => {
                let mut set = hashbrown::HashSet::with_capacity(*ptr);
                for item in arr.iter_mut().take(*ptr) {
                    unsafe {
                        set.insert(item.assume_init());
                    }
                }
                *self = Self::BrownSet(set)
            },
            Self::StdSet(_) => (),
            Self::BrownSet(_) => (),
        }
    }
}