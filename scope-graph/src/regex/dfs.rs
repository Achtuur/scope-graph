use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use crate::label::ScopeGraphLabel;

use super::Regex;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct AutomataNode<Lbl>
where
    Lbl: Clone + PartialEq + Eq + Hash,
{
    pub value: Regex<Lbl>,
    pub edges: Vec<(Lbl, usize)>,

}

impl<Lbl> AutomataNode<Lbl>
where
    Lbl: Clone + PartialEq + Eq + Hash,
{
    pub fn new(val: Regex<Lbl>) -> Self {
        Self {
            value: val,
            edges: Vec::new(),
        }
    }

    pub fn get_edge(&self, lbl: &Lbl) -> Option<&usize> {
        self.edges.iter()
        .find(|(l, _)| l == lbl)
        .map(|(_, idx)| idx)
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct RegexAutomata<Lbl>
where
    Lbl: Clone + PartialEq + Eq + Hash,
{
    pub node_vec: Vec<AutomataNode<Lbl>>,
    raw_reg: Regex<Lbl>,
}

impl<Lbl> RegexAutomata<Lbl>
where
    Lbl: Clone + PartialEq + Eq + Hash,
{
    /// Create a new automata from a regex, also compiles the regex
    pub fn from_regex(regex: Regex<Lbl>) -> Self {
        let mut automata = Self {
            node_vec: Vec::new(),
            raw_reg: regex.clone(),
        };
        automata.compile(regex);
        automata
    }

    pub fn is_empty(&self) -> bool {
        self.node_vec.is_empty()
    }

    pub fn get_node_mut(&mut self, regex: &Regex<Lbl>) -> Option<&mut AutomataNode<Lbl>> {
        self.node_vec
        .iter_mut()
        .find(|n| n.value == *regex)
    }

    pub fn get_node_idx(&self, regex: &Regex<Lbl>) -> Option<usize> {
        self.node_vec
        .iter()
        .position(|n| n.value == *regex)
    }

    pub fn is_match(&self, haystack: &[&Lbl]) -> bool {
        match self.match_haystack(haystack) {
            Some(node) => node.is_nullable(),
            None => false,
        }
    }

    pub fn partial_match(&self, haystack: &[&Lbl]) -> bool {
        self.match_haystack(haystack).is_some()
    }

    fn compile(&mut self, reg: Regex<Lbl>) {
        self.node_vec.push(AutomataNode::new(reg.clone()));
        let mut queue = vec![reg];

        while let Some(key) = queue.pop() {
            if matches!(key, Regex::EmptyString | Regex::ZeroSet) {
                continue;
            }

            let alfabet = key.leading_labels();
            for a in &alfabet {
                let derivative = key.derivative(a).reduce();

                // add new node if it doesn't exist
                if self.get_node_mut(&derivative).is_none() {
                    self.node_vec.push(AutomataNode::new(derivative.clone()));
                    queue.push(derivative.clone());
                }

                let derivative_idx = self.get_node_idx(&derivative).unwrap();
                let node = self.get_node_mut(&key).unwrap();
                node.edges.push(((*a).clone(), derivative_idx));

            }
        }
    }

    /// Traverses the DFA and returns the node where the search ends. If no match is found, returns None
    fn match_haystack<'a>(&'a self, haystack: &[&Lbl]) -> Option<&'a Regex<Lbl>> {
        if self.is_empty() {
            return None;
        }

        let mut current_node = &self.node_vec[0];

        for label in haystack {

            match current_node.get_edge(label) {
                Some(node_idx) => {
                    current_node = &self.node_vec[*node_idx]
                }
                None => {
                    return None;
                }
            }
        }
        Some(&current_node.value)
    }
}

impl<Lbl> RegexAutomata<Lbl>
where
    Lbl: Clone + PartialEq + Eq + Hash + std::fmt::Display,
{
    // uses display impl and removes spaces
    fn node_key(node_idx: usize) -> u64 {
        // let node_str = node.to_string();
        // node_str.replace(" ", "");
        let mut s = std::hash::DefaultHasher::new();
        node_idx.hash(&mut s);
        s.finish()
    }

    pub fn to_mmd(&self) -> String {
        let mut mmd = String::new();
        mmd += "---\ntitle: Regex Automata\n---\n";
        mmd += "flowchart LR\n";
        for (idx, node) in self.node_vec.iter().enumerate() {
            let node_key = Self::node_key(idx);
            mmd += &format!("{0:}(({1:}))\n", node_key, node.value);
        }

        for (idx, node) in self.node_vec.iter().enumerate() {
            for (lbl, target_node) in &node.edges {
                let node_key = Self::node_key(idx);
                let target_key = Self::node_key(*target_node);
                mmd += &format!("{0:} ==>|\"{1:}\"| {2:}\n", node_key, lbl, target_key);
            }
        }

        mmd
    }
}

impl<Lbl> std::fmt::Display for RegexAutomata<Lbl>
where Lbl: ScopeGraphLabel + std::fmt::Display + Clone + Eq + Hash
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_reg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_mmd_to_file(mmd: &str) {
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create("regex_automata.mmd").unwrap();
        file.write_all(mmd.as_bytes()).unwrap();
    }

    #[test]
    fn test_generate() {
        let regex = Regex::or(
            Regex::concat('a', 'c'),
            Regex::concat('b', 'c'),
        );

        // let regex = Regex::or(
        //     Regex::concat('a', 'c'),
        //     Regex::concat('b', 'c'),
        // );
        // let regex = Regex::kleene('a');
        // let regex = Regex::concat(Regex::kleene('P'), Regex::concat('P', 'D'));

        // let mut regex = Regex::from('a');
        // for c in 'b'..='z' {
        //     regex = Regex::concat(regex, c);
        // }

        let automata = RegexAutomata::from_regex(regex);
        let timer = std::time::Instant::now();
        println!("{:?}", timer.elapsed());
        let mmd = automata.to_mmd();
        write_mmd_to_file(&mmd);
    }

    #[test]
    fn test_is_match() {
        let regex = Regex::kleene('a');
        let automata = RegexAutomata::from_regex(regex);
        let mmd = automata.to_mmd();
        write_mmd_to_file(&mmd);
        let haystack = vec![&'a'; 10];
        assert!(automata.is_match(&haystack));
        let haystack = vec![&'b'];
        assert!(!automata.is_match(&haystack));
    }

    #[test]
    fn test_is_match_kleene() {
        let regex = Regex::concat(Regex::kleene('P'), Regex::concat('P', 'D'));
        let automata = RegexAutomata::from_regex(regex);
        let mmd = automata.to_mmd();
        write_mmd_to_file(&mmd);
        let haystack = vec![&'P', &'P', &'D'];
        assert!(automata.is_match(&haystack));
    }
}
