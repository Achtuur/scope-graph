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

fn queries_data() -> ParseResult<()> {
    let file = File::open(format!("{}/{}", BASE_PATH, QUERIES_FILE))?;
    let mut buf = BufReader::new(file);

    let timer = std::time::Instant::now();
    let mut deserializer = serde_json::Deserializer::from_reader(&mut buf);
    deserializer.disable_recursion_limit();
    // let json: serde_json::Value = Deserialize::deserialize(&mut deserializer)?;
    // println!("{:?}", timer.elapsed());

    // let arr = json.as_array().unwrap();

    // let first = arr[3].as_object().unwrap();
    // println!("first.keys().collect::<Vec<_>>()); {0:?}", first.keys().collect::<Vec<_>>());

    // let d = first.get("dataWf").unwrap();
    // println!("d: {0:#?}", d);

    let mut des: Vec<RawQueryData> = Deserialize::deserialize(&mut deserializer)?;
    let mut iter = des.iter().enumerate();
    let (_, first) = iter.next().unwrap(); // first is trivial, wildcard = wildcard
    let (_, eighth) = iter.nth(7).unwrap(); // eighth has 3 wildcard = 3 wildcards?
    for (idx, d) in iter {
        if first.dataOrd != d.dataOrd && eighth.dataOrd != d.dataOrd {
            println!("d: {0:#?}", idx);
        }
    }

    // for d in des.iter().take(5) {
    //     println!("d: {0:#?}", d.dataOrd);
    // }

    let mut d = des.get_mut(7871).unwrap();
    // let mut d = des.get_mut(7871).unwrap();

    d.dataWf.params.iter_mut().for_each(|wf| {
        wf.flatten_arrs();
    });

    println!("d: {0:#?}", d.dataWf);

    Ok(())
}

fn parsed_scopegraph_data() -> ParseResult<()> {
    let mut parsed_graph =
        ParsedScopeGraph::from_file(format!("{BASE_PATH}/{SCOPEGRAPH_FILE}"))?;

    println!("parsed_graph.len(): {0:?}", parsed_graph.scopes.len());
    parsed_graph.filter_scopes(|s| {
        !s.resource.contains("commons") || s.name.to_ascii_lowercase().contains("object")
    });
    println!("parsed_graph.len(): {0:?}", parsed_graph.scopes.len());
    parsed_graph.combine_scopes();
    println!("parsed_graph.len(): {0:?}", parsed_graph.scopes.len());
    parsed_graph.filter_edges(|e| {
        !matches!(
            e.label,
            JavaLabel::WithKind | JavaLabel::WithType | JavaLabel::LocalType
        )
    });
    // println!("parsed_graph.len(): {0:?}", parsed_graph.scopes.len());
    // parsed_graph.filter_edges(|e| matches!(e.label, JavaLabel::Extend | JavaLabel::Impl));

    // parsed_graph.filter_scope_by_edge_labels(|_, e_in, e_out| {
    //     let in_label = e_in.as_ref().map(|e| &e.label);
    //     let out_label = e_out.as_ref().map(|e| &e.label);

    //     matches!(in_label, Some(JavaLabel::Impl)) && matches!(out_label, Some(JavaLabel::Extend))
    // });
    println!("parsed_graph.len(): {0:?}", parsed_graph.scopes.len());
    parsed_graph.to_cosmograph_csv("output/cosmo.csv")?;
    println!("Written cosmo");

    return Ok(());

    let mut scope_vec = parsed_graph.scopes.keys().cloned().collect::<Vec<_>>();
    scope_vec.sort();
    let scope = scope_vec.get(2).cloned().unwrap();
    let full_graph = ScopeGraph::new(scope_vec, parsed_graph.edges);
    let mut partial_graph = get_scopegraph_section(&scope, &full_graph, 3);

    // partial_graph.filter_scopes(|s| !s.is_data());

    let mut style = PlantUmlStyleSheet::new();
    style.push(CssClass::new_class(
        "starting_scope".to_string(),
        ElementCss::new().background_color(Color::LIGHT_CYAN),
    ));

    let mut graph = PlantUmlDiagram::new("raw_data");
    graph.set_style_sheet(style);
    for s in full_graph.scopes {
        let mut item = PlantUmlItem::node(s.id(), &s.name, s.graph_node_type());
        if s == scope {
            item = item.add_class("starting_scope");
        }
        graph.push(item);
    }
    for e in full_graph.edges {
        let item = PlantUmlItem::edge(e.from.id(), e.to.id(), &e.label, EdgeDirection::Up);
        graph.push(item);
    }

    println!("graph.num_items(): {0:?}", graph.num_items());
    graph.render_to_file("output/parsed_graph.puml")?;

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
