use std::{
    cell::LazyCell, collections::{HashMap, HashSet}, fs::OpenOptions, io::{BufWriter, Write}, sync::{Arc, LazyLock}
};

use data_parse::{JavaLabel, ParsedScopeGraph};
use graphing::{
    plantuml::{EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem}, Color, Renderer
};
use serde::{Deserialize, Serialize};

use crate::pattern::Pattern;

pub mod pattern;
pub mod stat;

static TIMESTAMP: LazyLock<usize> = LazyLock::new(|| {
    let now = std::time::SystemTime::now();
    now.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
});


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scope(usize);

impl<'a> From<&'a Scope> for Scope {
    fn from(val: &'a Scope) -> Self {
        *val
    }
}

impl From<usize> for Scope {
    fn from(val: usize) -> Self {
        Scope(val)
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scope({})", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchableLabel {
    /// VarDecl, Method etc.
    ClassMember,
    Parent,
    ExtendImpl,
    Other,
}

impl std::fmt::Display for MatchableLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchableLabel::ClassMember => write!(f, "ClassMember"),
            MatchableLabel::Parent => write!(f, "Parent"),
            MatchableLabel::ExtendImpl => write!(f, "ExtendImpl"),
            MatchableLabel::Other => write!(f, "Other"),
        }
    }
}

impl From<JavaLabel> for MatchableLabel {
    fn from(value: JavaLabel) -> Self {
        match value {
            JavaLabel::VarDecl
            | JavaLabel::Method
            | JavaLabel::StaticMember => MatchableLabel::ClassMember,
            JavaLabel::StaticParent
            | JavaLabel::Parent => MatchableLabel::Parent,
            JavaLabel::Impl
            | JavaLabel::Extend => MatchableLabel::ExtendImpl,
            _ => MatchableLabel::Other,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    from: Scope,
    to: Scope,
    lbl: MatchableLabel,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScopeGraph {
    scopes: Vec<Scope>,
    edges: Vec<Edge>,
    /// map with scopes and incoming/outgoing edges
    from_edge_map: HashMap<Scope, Vec<Edge>>,
    to_edge_map: HashMap<Scope, Vec<Edge>>,
}

impl From<ParsedScopeGraph> for ScopeGraph {
    fn from(value: ParsedScopeGraph) -> Self {
        // let scopes = value.scopes
        // .into_keys()
        // .map(|s| Arc::from(s.name))
        // .map(Scope)
        // .collect::<Vec<_>>();

        let mut ctr = 0;
        let index_map = value.scopes.into_keys().fold(HashMap::new(), |mut acc, s| {
            acc.insert(s, Scope(ctr));
            ctr += 1;
            acc
        });

        let scopes = index_map.values().copied().collect::<Vec<_>>();

        let edges = value
            .edges
            .into_iter()
            .filter_map(|e| {
                let from = *index_map.get(&e.from)?;
                let to = *index_map.get(&e.to)?;
                Some(Edge {
                    from,
                    to,
                    lbl: e.label.into(),
                })
            })
            .collect::<Vec<_>>();

        let (from_edge_map, to_edge_map) = edges.iter()
        .fold((HashMap::new(), HashMap::new()), |(mut from_map, mut to_map), e| {
            from_map.entry(e.from).or_insert_with(Vec::new).push(e.clone());
            to_map.entry(e.to).or_insert_with(Vec::new).push(e.clone());
            (from_map, to_map)
        });

        ScopeGraph {
            scopes,
            edges,
            from_edge_map,
            to_edge_map,
        }
    }
}

impl ScopeGraph {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            edges: Vec::new(),
            from_edge_map: HashMap::new(),
            to_edge_map: HashMap::new(),
        }
    }

    pub fn add_node<S: Into<Scope>>(&mut self, node: S) {
        self.scopes.push(node.into());
    }

    pub fn add_edge<S: Into<Scope>>(&mut self, from: S, to: S) {
        self.add_edge_labeled(from, to, MatchableLabel::Other);
    }

    pub fn add_edge_labeled<S: Into<Scope>, L: Into<MatchableLabel>>(&mut self, from: S, to: S, lbl: L) {
        let edge = Edge {
            from: from.into(),
            to: to.into(),
            lbl: lbl.into(),
        };
        self.from_edge_map
            .entry(edge.from)
            .or_default()
            .push(edge.clone());
        self.to_edge_map.entry(edge.to).or_default().push(edge.clone());
        self.edges.push(edge);
    }

    pub fn get_outgoing_edges_with_labels(&self, s: impl Into<Scope>, lbls: &[MatchableLabel]) -> impl Iterator<Item =&Edge> {
        let s = s.into();
        self.from_edge_map
            .get(&s)
            .into_iter()
            // .flatten()
            .flat_map(|edges| {
                edges.iter().filter(|e| lbls.contains(&e.lbl))
            })
    }

    pub fn get_incoming_edges_with_labels(&self, s: impl Into<Scope>, lbls: &[MatchableLabel]) -> impl Iterator<Item = &Edge> {
        let s = s.into();
        self.to_edge_map
            .get(&s)
            .into_iter()
            // .flatten()
            .flat_map(|edges| {
                edges.iter().filter(|e| lbls.contains(&e.lbl))
            })
    }

    pub fn from_edges<S: Into<Scope>, L: Into<MatchableLabel>>(edges: impl IntoIterator<Item = (S, L, S)>) -> Self
    {
        let mut graph = Self::new();
        for (from, l, to) in edges {
            let (from, to) = (from.into(), to.into());
            if !graph.scopes.contains(&from) {
                graph.add_node(from);
            }

            if !graph.scopes.contains(&to) {
                graph.add_node(to);
            }
            graph.add_edge_labeled(from, to, l);
        }
        graph
    }

    pub fn match_subgraph(&self, pattern: &Pattern, name: &str) -> Vec<Vec<vf2::NodeIndex>> {
        let base_path = format!("output/patterns/{}/{}", name, *TIMESTAMP);
        std::fs::create_dir_all(&base_path).unwrap();

        let scope_graph_file = format!("{}/graph.json", base_path);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&scope_graph_file)
            .unwrap();
        let mut bufwriter = BufWriter::new(file);
        serde_json::to_writer(&mut bufwriter, &self).unwrap();

        let fname = format!("{}/{}.txt", base_path, pattern.file_name());
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(fname)
            .unwrap();
        let mut bufwriter = BufWriter::new(file);

        let pattern_graph = pattern.subgraph();
        let vf2 = vf2::induced_subgraph_isomorphisms(&pattern_graph, self);

        pattern
            .prune_matches(vf2.iter())
            // .inspect(|v| println!("match: {:?}", v))
            .inspect(|v| {
                let _ = bufwriter.write(format!("{:?},\n", v).as_bytes()).unwrap();
            })
            .collect()
    }

    pub fn diagram(&self) -> PlantUmlDiagram {
        let mut diagram = PlantUmlDiagram::new("graph");
        for node in &self.scopes {
            diagram.push(PlantUmlItem::node(node, node, NodeType::Node));
        }
        for e in &self.edges {
            diagram.push(PlantUmlItem::edge(e.from, e.to, &e.lbl, EdgeDirection::Up));
        }
        diagram
    }

    pub fn find_edge<S: Into<Scope>>(&self, from: S, to: S) -> Option<&Edge> {
        let (from, to) = (from.into(), to.into());
        let from_edges = self.from_edge_map.get(&from)?;
        let to_edges = self.to_edge_map.get(&to)?;

        match from_edges.len() < to_edges.len() {
            true => from_edges.iter().find(|e| e.to == to),
            false => to_edges.iter().find(|e| e.from == from),
        }
    }
}

impl vf2::Graph for ScopeGraph {
    type NodeLabel = Scope;

    type EdgeLabel = MatchableLabel;

    fn is_directed(&self) -> bool {
        true
    }

    fn node_count(&self) -> usize {
        self.scopes.len()
    }

    fn node_label(&self, node: vf2::NodeIndex) -> Option<&Self::NodeLabel> {
        self.scopes.get(node)
    }

    fn neighbors(
        &self,
        node: vf2::NodeIndex,
        direction: vf2::Direction,
    ) -> impl Iterator<Item = vf2::NodeIndex> {
        self.from_edge_map
            .get(&Scope(node))
            .into_iter()
            .flat_map(move |edges| {
                edges.iter().map(move |e| match direction {
                    vf2::Direction::Outgoing => e.to.0,
                    vf2::Direction::Incoming => e.from.0,
                })
            })
    }

    fn contains_edge(&self, source: vf2::NodeIndex, target: vf2::NodeIndex) -> bool {
        self.find_edge(source, target).is_some()
    }

    fn edge_label(
        &self,
        source: vf2::NodeIndex,
        target: vf2::NodeIndex,
    ) -> Option<&Self::EdgeLabel> {
        println!("edge_label: source: {}, target: {}", source, target);
        self.find_edge(source, target).map(|e| &e.lbl)
    }
}