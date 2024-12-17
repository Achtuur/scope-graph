use std::{collections::{HashMap, HashSet}, hash::{self, Hash, Hasher}};

use crate::label::ScopeGraphLabel;

use super::Regex;

pub trait RegexLabel: Default + Clone + PartialEq + Eq + Hash {}

impl<T: ScopeGraphLabel + Hash + Eq + Clone + Default> RegexLabel for T {}


#[derive(Default, Debug)]
pub struct AutomataNode<Lbl>
where Lbl: Clone + PartialEq + Eq + Hash
{
    // pub key: Regex<Lbl>,
    // key = edge label, value = target node key
    pub edges: HashMap<Lbl, Regex<Lbl>>,
}

impl<Lbl> AutomataNode<Lbl>
where Lbl: Clone + PartialEq + Eq + Hash
{
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct RegexAutomata<Lbl>
where Lbl: Clone + PartialEq + Eq + Hash

{
    pub start_key: Regex<Lbl>,
    pub nodes: HashMap<Regex<Lbl>, AutomataNode<Lbl>>,
}

impl<Lbl> RegexAutomata<Lbl>
where Lbl: Clone + PartialEq + Eq + Hash
{
    /// Create a new automata from a regex, also compiles the regex
    pub fn from_regex(regex: Regex<Lbl>) -> Self {
        let mut transitions = HashMap::new();
        transitions.insert(regex.clone(), AutomataNode::new());
        let mut automata = Self {
            start_key: regex.clone(),
            nodes: transitions,
        };
        automata.compile();
        automata
    }

    fn compile(&mut self) {
        let mut queue = vec![self.start_key.clone()];
        // let alfabet = self.start_key.unique_labels();
        while let Some(key) = queue.pop() {
            if matches!(key, Regex::EmptyString | Regex::ZeroSet) {
                continue;
            }

            // let alfabet = key.unique_labels();
            let alfabet = key.leading_labels();
            // take derivative wrt to each character in alfabet
            // if new state -> add to queue
            for a in &alfabet {
                // compute derivative
                let derivative = key.derivative(a).reduce();
                // add node if it doesnt exist yet
                if !self.nodes.contains_key(&derivative) {
                    self.nodes.insert(derivative.clone(), AutomataNode::new());
                    queue.push(derivative.clone());
                }
                // add edge to new node
                let node = self.nodes.get_mut(&key).unwrap();
                node.edges.insert((*a).clone(), derivative);
            }
        }
    }

    /// Traverses the DFA and returns the node where the search ends. If no match is found, returns None
    fn match_haystack<'a>(&'a self, haystack: &[&Lbl]) -> Option<&'a Regex<Lbl>> {
        let mut current = &self.start_key;
        for label in haystack {
            let Some(node) = self.nodes.get(current) else {
                // no node found with current key (should be impossible i think)
                return None;
            };

            match node.edges.get(label) {
                Some(e) => current = e,
                None => {
                    // no edge found with this `label` -> no match
                    return None;
                },
            }
        }
        Some(current)
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
}

impl<Lbl> RegexAutomata<Lbl>
where Lbl: Clone + PartialEq + Eq + Hash + std::fmt::Display
{
    // uses display impl and removes spaces
    fn node_key(node: &Regex<Lbl>) -> u64 {
        // let node_str = node.to_string();
        // node_str.replace(" ", "");
        let mut s = std::hash::DefaultHasher::new();
        node.hash(&mut s);
        s.finish()
    }

    pub fn to_mmd(&self) -> String {
        let mut mmd = String::new();
        mmd += "---\ntitle: Regex Automata\n---\n";
        mmd += "flowchart LR\n";
        for node in self.nodes.keys() {
            let node_key = Self::node_key(node);
            mmd += &format!("{node_key}(({node}))\n",)
        }

        for (node_reg, node) in &self.nodes {

            // group node.edges by target node
            let mut grouped_edges = HashMap::new();
            for (label, target) in &node.edges {
                let entry = grouped_edges.entry(target).or_insert_with(Vec::new);
                entry.push(label);
            }

            for (target, labels) in grouped_edges {
                let node_key = Self::node_key(node_reg);
                let target_key = Self::node_key(target);
                let mut combined_label = String::new();
                for l in labels {
                    combined_label += &format!("{0:}, ", l);
                }
                mmd += &format!(
                    "{0:} ==>|\"{1:}\"| {2:}\n",
                    node_key, combined_label.trim_end_matches(", "), target_key
                );
            }
        }

        mmd
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
        // let regex = Regex::or(
        //     Regex::concat('a', 'c'),
        //     Regex::concat('b', 'c'),
        // );

        // let regex = Regex::or(
        //     Regex::concat('a', 'c'),
        //     Regex::concat('b', 'c'),
        // );
        // let regex = Regex::kleene('a');
        let regex = Regex::concat(Regex::kleene('P'), Regex::concat('P','D'));

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
        let regex = Regex::concat(Regex::kleene('P'), Regex::concat('P','D'));
        let mut automata = RegexAutomata::from_regex(regex);
        let mmd = automata.to_mmd();
        write_mmd_to_file(&mmd);
        let haystack = vec![&'P', &'D'];
        assert!(automata.is_match(&haystack));
    }
}