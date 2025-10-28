use crate::label::ScopeGraphLabel;

use super::dfs::RegexAutomaton;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegexState<'a, Lbl>
where
    Lbl: ScopeGraphLabel,
{
    automata: &'a RegexAutomaton<Lbl>,
    prev_idx: usize,
    idx: usize,
}

impl<'a, Lbl> RegexState<'a, Lbl>
where
    Lbl: ScopeGraphLabel,
{
    #[inline]
    pub fn new(automata: &'a RegexAutomaton<Lbl>) -> Self {
        Self {
            automata,
            idx: 0,
            prev_idx: 0,
        }
    }

    #[inline]
    pub fn with_index(automata: &'a RegexAutomaton<Lbl>, idx: usize) -> Self {
        Self {
            automata,
            idx,
            prev_idx: 0,
        }
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.idx
    }

    pub fn prev_index(&self) -> usize {
        self.prev_idx
    }

    /// Steps through the automata, returning the next node index.
    ///
    /// Returns `None` if the step does not exist.
    pub fn step(&mut self, label: &Lbl) -> Option<usize> {
        let node = self.automata.get_node(self.idx)?;
        let next_idx = node.get_edge(label)?;
        self.prev_idx = self.idx;
        self.idx = *next_idx;
        Some(self.idx)
    }

    #[inline]
    pub fn is_accepting(&self) -> bool {
        self.automata
            .get_node(self.idx)
            .is_some_and(|node| node.value.is_nullable())
    }
}
