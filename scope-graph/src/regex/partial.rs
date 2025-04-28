use crate::label::ScopeGraphLabel;

use super::dfs::RegexAutomata;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartialRegex<'a, Lbl>
where
    Lbl: ScopeGraphLabel,
{
    automata: &'a RegexAutomata<Lbl>,
    idx: usize,
}

impl<'a, Lbl> PartialRegex<'a, Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub fn new(automata: &'a RegexAutomata<Lbl>) -> Self {
        Self { automata, idx: 0 }
    }

    pub fn with_index(automata: &'a RegexAutomata<Lbl>, idx: usize) -> Self {
        Self { automata, idx }
    }

    pub fn index(&self) -> usize {
        self.idx
    }

    /// Steps through the automata, returning the next node index.
    ///
    /// Returns `None` if the step does not exist.
    pub fn step(&mut self, label: &Lbl) -> Option<usize> {
        let node = self.automata.get_node(self.idx)?;
        let next_idx = node.get_edge(label)?;
        self.idx = *next_idx;
        Some(self.idx)
    }

    pub fn is_accepting(&self) -> bool {
        self.automata
            .get_node(self.idx)
            .is_some_and(|node| node.value.is_nullable())
    }
}
