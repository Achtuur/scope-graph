#![allow(unused)]
use std::{collections::{HashMap, HashSet}, fs::{File, OpenOptions}, io::BufReader};

mod raw;
mod parsed;
mod error;

pub use error::*;
use graphing::{mermaid::{item::{ItemShape, MermaidItem}, theme::EdgeType, MermaidDiagram}, plantuml::{EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem}, Renderer};
use serde::Deserialize;

use crate::{parsed::{ParsedEdge, ParsedLabel, ParsedScope}, raw::{JavaType, JavaValue, RawEdge, RawLabel, RawQueryData, RawScope, RawScopeGraph, RefType}};

const BASE_PATH: &str = "./raw/";
const QUERIES_FILE: &str = "commons-csv.queries.json";
const RESULTS_FILE: &str = "commons-csv.results.json";
const SCOPEGRAPH_FILE: &str = "commons-csv.scopegraph.json";

fn main() -> ParseResult<()> {
    // queries_data()?;
    scopegraph_data()?;
    Ok(())
}

fn queries_data() -> ParseResult<()> {
    let file = File::open(format!("{}{}", BASE_PATH, QUERIES_FILE))?;
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

fn scopegraph_data() -> ParseResult<()> {
    let file =
        File::open(format!("{}{}", BASE_PATH, SCOPEGRAPH_FILE))?;
    let mut buf = BufReader::new(file);

    let timer = std::time::Instant::now();
    let mut deserializer = serde_json::Deserializer::from_reader(&mut buf);
    deserializer.disable_recursion_limit();
    let mut json: RawScopeGraph = Deserialize::deserialize(&mut deserializer)?;
    println!("{:?}", timer.elapsed());

    println!("json.data.len(): {0:?}", json.data.len());
    println!("json.edges.len(): {0:?}", json.edges.len());
    println!("json.labels.len(): {0:?}", json.labels.len());

    let parsed_edges = json.edges.into_iter()
    .flat_map(|(key, edge)| {
        ParsedEdge::from_raw(key, RawEdge::Head(edge))
    })
    .collect::<Vec<_>>();


    // take 1 edge, get the scopes -> get edges with that scope -> repeat


    let mut scopes = json.data.into_keys()
    .map(|s| ParsedScope::new(&s))
    .collect::<Vec<_>>();


    let scope = scopes.get(160).unwrap();
    let (scopes, relevant_edges) = get_scopegraph_section(scope, &scopes, &parsed_edges, 5, 0);

    // let scope = scopes.get(150).unwrap();
    // let (scopes, relevant_edges) = get_scopegraph_section(scope, &scopes, &parsed_edges, 5);

    // let scopes = json.data.into_keys().take(25).collect::<Vec<_>>();
    // let relevant_edges = parsed_edges.into_iter().filter(|e| {
    //     scopes.contains(&e.from) || scopes.contains(&e.to)
    // })
    // .collect::<Vec<_>>();

    let mut graph = PlantUmlDiagram::new("raw data");
    for s in scopes {
        let item = PlantUmlItem::node(&s.name, &s.name, NodeType::Node);
        // let item = MermaidItem::node(&s.name, &s.name, ItemShape::Circle);
        graph.push(item);
    }

    for e in relevant_edges {
        let item = MermaidItem::edge(&e.from, &e.to, &e.label, EdgeType::Solid);
        let item = PlantUmlItem::edge(&e.from, &e.to, &e.label, EdgeDirection::Unspecified);
        graph.push(item);
    }

    graph.render_to_file("output/parsed_graph.puml")?;
    Ok(())
}

const MAX_DEPTH: usize = 15;
fn get_scopegraph_section(scope: &ParsedScope, scopes: &[ParsedScope], edges: &[ParsedEdge], size: usize, depth: usize) -> (Vec<ParsedScope>, Vec<ParsedEdge>) {
    // take a scope and find all edges that connect to it
    // recursively find all those scopes and do the same

    if depth > MAX_DEPTH {
        return (Vec::new(), Vec::new())
    }

    let mut new_scopes = vec![scope.clone()];
    let mut found_edges = Vec::new();
    let adj_edges = edges.iter().filter(|e| e.from == scope.name || e.to == scope.name);
    for edge in adj_edges {
        let other_scope = if edge.from == scope.name {
            scopes.iter().find(|s| s.name == edge.to)
        } else {
            scopes.iter().find(|s| s.name == edge.from)
        };

        if other_scope.is_none() {
            continue;
        }
        let (child_scopes, child_edges) = get_scopegraph_section(other_scope.unwrap(), scopes, edges, size, depth + 1);
        found_edges.push(edge.clone());
        found_edges.extend(child_edges);
        new_scopes.extend(child_scopes);
        if new_scopes.len() >= size {
            break;
        }
    }
    (new_scopes, found_edges)
}