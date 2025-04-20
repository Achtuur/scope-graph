use std::{collections::VecDeque, path::Path};

use crate::label::ScopeGraphLabel;

/// A reduced path is a paht that has been reduced to ignore repeating labels in paths.
///
/// Ie, `PPPD` is reduced to `PD`, `ABCD` is reduced to `ABCD`
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct ReducedPath<Lbl: ScopeGraphLabel>
{
    path: VecDeque<Lbl>,
}

impl<Lbl: ScopeGraphLabel> std::fmt::Display for ReducedPath<Lbl>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.iter().map(|l| l.to_string()).collect::<String>())
    }
}

impl<Lbl: ScopeGraphLabel> ReducedPath<Lbl>
{
    pub fn new() -> Self {
        Self {
            path: VecDeque::new(),
        }
    }

    pub fn start(label: Lbl) -> Self {
        let mut path = Self::new();
        path.push(label);
        path
    }

    pub fn peek(&self) -> Option<&Lbl> {
        self.path.front()
    }

    pub fn push(&mut self, label: Lbl) {
        if self.should_push(&label) {
            self.path.push_front(label);
        }
    }

    pub fn should_push(&self, label: &Lbl) -> bool {
        self.path.is_empty() || self.peek() != Some(label)
    }

    /// Returns true if `self` is the same as `other` prepended with `start`
    pub fn is_same_path_with_start(&self, start: &Lbl, other: &Self) -> bool {
        // // they can never be equal if they are not the same length
        // if self.path.len() != other.path.len() + 1 {
        //     return false;
        // }

        if other.should_push(start) {
            // if 'start' should be appened to 'other', append the iters
            let other_iter = std::iter::once(start).chain(other.path.iter());
            for (lbl1, lbl2) in self.path.iter().zip(other_iter) {
                if lbl1 != lbl2 {
                    return false;
                }
            }
            true
        } else {
            // else we can just compare the paths
            self.path == other.path
        }
    }

    pub fn as_lbl_vec(&self) -> impl Iterator<Item = &Lbl> {
        self.path.iter()
        // self.path.as_slices().0
    }
}