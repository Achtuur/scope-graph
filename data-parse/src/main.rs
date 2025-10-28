#![allow(unused)]
use data_parse::*;
use graphing::{
    Color, Renderer,
    mermaid::{
        MermaidDiagram,
        item::{ItemShape, MermaidItem},
        theme::EdgeType,
    },
    plantuml::{
        EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem,
        theme::{CssClass, ElementCss, PlantUmlStyleSheet},
    },
};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    str::FromStr,
};

const BASE_PATH: &str = "./raw";
const QUERIES_FILE: &str = "commons-csv-queries.json";
const RESULTS_FILE: &str = "commons-csv-results.json";
const SCOPEGRAPH_FILE: &str = "commons-io-scopegraph.json";

fn main() -> ParseResult<()> {
    // queries_data()?;
    // scopegraph_data()?;
    parsed_scopegraph_data()?;
    Ok(())
}

// attempt to parse query data; unsucessful
fn queries_data() -> ParseResult<()> {
    let file = File::open(format!("{}/{}", BASE_PATH, QUERIES_FILE))?;
    let mut buf = BufReader::new(file);

    let timer = std::time::Instant::now();
    let mut deserializer = serde_json::Deserializer::from_reader(&mut buf);
    deserializer.disable_recursion_limit();

    let mut des: Vec<RawQueryData> = Deserialize::deserialize(&mut deserializer)?;
    let mut iter = des.iter().enumerate();
    let (_, first) = iter.next().unwrap(); // first is trivial, wildcard = wildcard
    let (_, eighth) = iter.nth(7).unwrap(); // eighth has 3 wildcard = 3 wildcards?
    for (idx, d) in iter {
        if first.dataOrd != d.dataOrd && eighth.dataOrd != d.dataOrd {
            println!("d: {0:#?}", idx);
        }
    }

    let mut d = des.get_mut(7871).unwrap();
    // let mut d = des.get_mut(7871).unwrap();

    d.dataWf.params.iter_mut().for_each(|wf| {
        wf.flatten_arrs();
    });

    println!("d: {0:#?}", d.dataWf);

    Ok(())
}

fn parsed_scopegraph_data() -> ParseResult<()> {
    let mut parsed_graph = ParsedScopeGraph::from_file(format!("{BASE_PATH}/{SCOPEGRAPH_FILE}"))?;

    println!("Filtering scope graph for stdlib scopes only...");
    // parsed_graph.filter_scopes(|s| {
    //     !s.resource.contains("commons") || s.name.to_ascii_lowercase().contains("object")
    // });
    // parsed_graph.combine_scopes();
    // parsed_graph.filter_edges(|e| {
    //     !matches!(
    //         e.label,
    //         JavaLabel::WithKind | JavaLabel::WithType | JavaLabel::LocalType
    //     )
    // });
    println!("parsed_graph.len(): {0:?}", parsed_graph.scopes.len());
    std::fs::create_dir_all("./output/")?;
    parsed_graph.to_cosmograph_csv("./output/cosmo.csv")?;
    println!("Written scope graph to output/cosmo.csv");
    Ok(())
}

#[derive(Default)]
struct ScopeGraph {
    scopes: HashSet<ParsedScope>,
    edges: HashSet<ParsedEdge>,
}

impl ScopeGraph {
    pub fn new(
        scope: impl IntoIterator<Item = ParsedScope>,
        edges: impl IntoIterator<Item = ParsedEdge>,
    ) -> Self {
        Self {
            scopes: scope.into_iter().collect(),
            edges: edges.into_iter().collect(),
        }
    }

    pub fn combine(&mut self, other: ScopeGraph) {
        self.scopes.extend(other.scopes);
        self.edges.extend(other.edges);
    }

    pub fn filter_scopes(&mut self, filter: impl Fn(&ParsedScope) -> bool) {
        self.scopes.retain(&filter);
        self.edges.retain(|e| filter(&e.from) && filter(&e.to));
    }
}

fn get_scopegraph_section(
    scope: &ParsedScope,
    full_graph: &ScopeGraph,
    depth: usize,
) -> ScopeGraph {
    let mut graph = ScopeGraph::default();
    graph.scopes.insert(scope.clone());
    // queue with scopes that are reached with a "from" edge
    let mut scope_queue = HashSet::from([scope]);

    for _ in 0..depth {
        // consume entire queue
        let mut new_queue = HashSet::new();
        for cur_scope in scope_queue {
            let adj_edges = full_graph
                .edges
                .iter()
                .filter(|e| &e.from == cur_scope || &e.to == cur_scope);
            for edge in adj_edges {
                graph.scopes.insert(edge.from.clone());
                graph.scopes.insert(edge.to.clone());
                graph.edges.insert(edge.clone());
                if &edge.from != cur_scope {
                    new_queue.insert(&edge.from);
                }
                if &edge.to != cur_scope {
                    new_queue.insert(&edge.to);
                }
            }
        }
        scope_queue = new_queue;
    }
    graph
}
