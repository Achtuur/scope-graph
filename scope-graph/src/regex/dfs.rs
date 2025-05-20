use std::hash::Hash;

use graphing::{
    mermaid::{
        MermaidDiagram,
        item::{ItemShape, MermaidItem},
        theme::EdgeType,
    },
    plantuml::{EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem},
};

use crate::label::ScopeGraphLabel;

use super::Regex;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AutomatonNode<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub value: Regex<Lbl>,
    pub edges: Vec<(Lbl, usize)>,
}

impl<Lbl> AutomatonNode<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub fn new(val: Regex<Lbl>) -> Self {
        Self {
            value: val,
            edges: Vec::new(),
        }
    }

    pub fn get_edge(&self, lbl: &Lbl) -> Option<&usize> {
        self.edges
            .iter()
            .find(|(l, _)| l == lbl)
            .map(|(_, idx)| idx)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RegexAutomaton<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    node_vec: Vec<AutomatonNode<Lbl>>,
    raw_reg: Regex<Lbl>,
}

impl<Lbl> RegexAutomaton<Lbl>
where
    Lbl: ScopeGraphLabel,
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

    fn compile(&mut self, reg: Regex<Lbl>) {
        self.node_vec.push(AutomatonNode::new(reg.clone()));
        let mut queue = vec![reg];

        while let Some(key) = queue.pop() {
            if matches!(key, Regex::EmptyString | Regex::ZeroSet) {
                continue;
            }

            let alfabet = key.leading_labels();
            // println!("(key, alfabet): {0:?}", (&key, &alfabet));
            for a in &alfabet {
                let derivative = key.derivative(a).reduce();

                // add new node if it doesn't exist
                let derivative_idx = if self.get_node_mut(&derivative).is_none() {
                    queue.push(derivative.clone());
                    self.node_vec.push(AutomatonNode::new(derivative));
                    self.node_vec.len() - 1
                } else {
                    self.get_node_idx(&derivative).unwrap()
                };

                let node = self.get_node_mut(&key).unwrap();
                node.edges.push(((*a).clone(), derivative_idx));
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.node_vec.is_empty()
    }

    pub fn get_node(&self, idx: usize) -> Option<&AutomatonNode<Lbl>> {
        self.node_vec.get(idx)
    }

    pub fn get_node_mut(&mut self, regex: &Regex<Lbl>) -> Option<&mut AutomatonNode<Lbl>> {
        self.node_vec.iter_mut().find(|n| n.value == *regex)
    }

    pub fn get_node_idx(&self, regex: &Regex<Lbl>) -> Option<usize> {
        self.node_vec.iter().position(|n| n.value == *regex)
    }

    pub fn is_match<'a>(&'a self, haystack: impl IntoIterator<Item = &'a Lbl>) -> bool {
        let Some(node) = self.match_haystack(haystack) else {
            return false;
        };
        node.is_nullable()
    }

    pub fn partial_match<'a>(&'a self, haystack: impl IntoIterator<Item = &'a Lbl>) -> bool {
        self.match_haystack(haystack).is_some()
    }

    pub fn index_of<'a>(&'a self, haystack: impl IntoIterator<Item = &'a Lbl>) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        let mut current_node = &self.node_vec[0];
        let mut index = 0;

        for label in haystack {
            match current_node.get_edge(label) {
                Some(node_idx) => {
                    current_node = &self.node_vec[*node_idx];
                    index += 1;
                }
                None => {
                    return None;
                }
            }
        }
        Some(index)
    }

    /// Traverses the DFA and returns the node where the search ends. If no match is found, returns None
    fn match_haystack<'a>(
        &'a self,
        haystack: impl IntoIterator<Item = &'a Lbl>,
    ) -> Option<&'a Regex<Lbl>> {
        if self.is_empty() {
            return None;
        }

        let mut current_node = &self.node_vec[0];

        for label in haystack {
            match current_node.get_edge(label) {
                Some(node_idx) => current_node = &self.node_vec[*node_idx],
                None => {
                    return None;
                }
            }
        }
        Some(&current_node.value)
    }
}

impl<Lbl> RegexAutomaton<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    // uses display impl and removes spaces
    fn node_key(node_idx: usize) -> String {
        format!("n{}", node_idx)
    }

    pub fn to_mmd(&self) -> MermaidDiagram {
        let mut diagram = MermaidDiagram::new("Regex Automata");

        let nodes = self.node_vec.iter().enumerate().map(|(idx, node)| {
            MermaidItem::node(
                Self::node_key(idx),
                node.value.to_string(),
                ItemShape::Rounded,
            )
        });

        let edges = self.node_vec.iter().enumerate().flat_map(|(idx, node)| {
            let from = Self::node_key(idx);
            node.edges.iter().map(move |(lbl, target_idx)| {
                let to = Self::node_key(*target_idx);

                MermaidItem::edge(&from, to, lbl.to_string(), EdgeType::Solid)
            })
        });

        diagram.extend(nodes);
        diagram.extend(edges);

        diagram
    }

    pub fn to_uml(&self) -> PlantUmlDiagram {
        let mut diagram = PlantUmlDiagram::new("Regex Automata");

        let nodes = self.node_vec.iter().enumerate().map(|(idx, node)| {
            PlantUmlItem::node(Self::node_key(idx), node.value.to_string(), NodeType::Node)
        });

        let edges = self.node_vec.iter().enumerate().flat_map(|(idx, node)| {
            let from = Self::node_key(idx);
            node.edges.iter().map(move |(lbl, target_idx)| {
                let to = Self::node_key(*target_idx);
                let dir = if from == to {
                    EdgeDirection::Right
                } else {
                    EdgeDirection::Unspecified
                };

                PlantUmlItem::edge(&from, to, lbl.to_string(), dir)
            })
        });

        diagram.extend(nodes);
        diagram.extend(edges);

        diagram
    }
}

impl<Lbl> std::fmt::Display for RegexAutomaton<Lbl>
where
    Lbl: ScopeGraphLabel + std::fmt::Display + Clone + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_reg)
    }
}

#[cfg(test)]
mod tests {
    use graphing::Renderer;

    use super::*;

    #[test]
    fn test_generate() {
        // let regex = Regex::or(Regex::concat('a', 'c'), Regex::concat('b', 'c'));

        // let regex: Regex<char> = Regex::neg(Regex::ZeroSet);
        let regex: Regex<char> = Regex::kleene(Regex::or('p', 'q'));

        let regex: Regex<char> = Regex::concat(
            Regex::question('a'),
            Regex::concat('b', Regex::question('c')),
        );

        let automata = RegexAutomaton::from_regex(regex);
        let timer = std::time::Instant::now();
        automata
        .to_uml()
        .render_to_file("output/regex/automata.puml")
        .unwrap();
        println!("{:?}", timer.elapsed());
    }

    #[test]
    fn test_is_match() {
        let regex = Regex::kleene('a');
        let automata = RegexAutomaton::from_regex(regex);
        automata
            .to_mmd()
            .render_to_file("output/regex/automata.md")
            .unwrap();
        let haystack = vec!['a'; 10];
        assert!(automata.is_match(&haystack));
        let haystack = vec!['b'];
        assert!(!automata.is_match(&haystack));
    }

    #[test]
    fn test_is_match_kleene() {
        let regex = Regex::concat(Regex::kleene('P'), Regex::concat('P', 'D'));
        let automata = RegexAutomaton::from_regex(regex);
        automata
            .to_uml()
            .render_to_file("output/regex/automata.md")
            .unwrap();
        let haystack = vec!['P', 'P', 'D'];
        assert!(automata.is_match(&haystack));
    }
}
