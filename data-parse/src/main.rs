#![allow(unused)]
use std::{collections::{HashMap, HashSet}, fs::{File, OpenOptions}, io::{BufReader, BufWriter}, str::FromStr};

mod raw;
mod parsed;
mod error;

pub use error::*;
use graphing::{mermaid::{item::{ItemShape, MermaidItem}, theme::EdgeType, MermaidDiagram}, plantuml::{EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem}, Renderer};
use serde::Deserialize;

use crate::{parsed::{ParsedEdge, ParsedLabel, ParsedScope, ParsedScopeGraph, ScopeData}, raw::{JavaType, JavaValue, RawEdge, RawLabel, RawQueryData, RawScope, RawScopeGraph, RefType}};

const BASE_PATH: &str = "./raw";
const QUERIES_FILE: &str = "commons-csv.queries.json";
const RESULTS_FILE: &str = "commons-csv.results.json";
const SCOPEGRAPH_FILE: &str = "commons-csv.scopegraph.json";

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
    // slightly faster to deserialize
    let file =
        File::open(format!("{}/{}", BASE_PATH, "scopegraph_cache.json"))?;
    let mut buf = BufReader::new(file);

    let timer = std::time::Instant::now();
    let mut deserializer = serde_json::Deserializer::from_reader(&mut buf);
    deserializer.disable_recursion_limit();
    let mut json: ParsedScopeGraph = Deserialize::deserialize(&mut deserializer)?;
    println!("{:?}", timer.elapsed());


    // scopes that hold a reference to scope_550
    let scope_550_ref = json.scopes.iter()
        .filter(|(_, data)| matches!(data, ScopeData::Ref(s) if s.name == "s_ty-550"))
        .map(|(s, _)| s)
        .collect::<Vec<_>>();

    let edge_to_scope_550 = json.edges.iter()
        .filter(|e| scope_550_ref.contains(&&e.to))
        .fold(HashMap::new(), |mut acc, e| {
            let occurences: &mut usize = acc.entry(&e.label).or_default();
            *occurences += 1;
            acc
        });

    println!("edge_to_scope_550: {0:#?}", edge_to_scope_550);

    let x = json.edges.iter()
    .filter(|e| e.label.contains("return"))
    .find(|e| scope_550_ref.contains(&&e.to));

    println!("x: {0:#?}", x);

    // let scopes = json.scopes.keys().take(1000).collect::<Vec<_>>();
    // let edges = json.edges.iter().filter(|e| scopes.contains(&&e.from) || scopes.contains(&&e.to)).collect::<Vec<_>>();


    let mut scope_vec = json.scopes.keys().cloned().collect::<Vec<_>>();
    scope_vec.sort();
    let scope = scope_vec.get(550).unwrap(); 
    let (scopes, relevant_edges) = get_scopegraph_section(scope, &scope_vec, &json.edges, 200, 0);

    let mut graph = PlantUmlDiagram::new("raw_data");
    for e in relevant_edges {
        let from = PlantUmlItem::node(e.from.id(), &e.from.name, NodeType::Node);
        let to = PlantUmlItem::node(e.to.id(), &e.to.name, NodeType::Node);
        let item = PlantUmlItem::edge(e.from.id(), e.to.id(), &e.label, EdgeDirection::Up);
        graph.push(from);
        graph.push(to);
        graph.push(item);
    }


    println!("graph.num_items(): {0:?}", graph.num_items());

    graph.render_to_file("output/parsed_graph.puml")?;

    Ok(())
}

fn scopegraph_data() -> ParseResult<()> {
    let file =
        File::open(format!("{}/{}", BASE_PATH, SCOPEGRAPH_FILE))?;
    let mut buf = BufReader::new(file);

    let timer = std::time::Instant::now();
    let mut deserializer = serde_json::Deserializer::from_reader(&mut buf);
    deserializer.disable_recursion_limit();
    let mut json: RawScopeGraph = Deserialize::deserialize(&mut deserializer)?;
    println!("{:?}", timer.elapsed());


    let parsed_graph = ParsedScopeGraph::try_from(json)?;
    let mut cache_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(format!("{}/{}", BASE_PATH, "scopegraph_cache.json"))?;
    let mut buf = BufWriter::new(cache_file);
    serde_json::to_writer_pretty(&mut buf, &parsed_graph)?;

    // println!("json.data.len(): {0:?}", json.data.len());
    // println!("json.edges.len(): {0:?}", json.edges.len());
    // println!("json.labels.len(): {0:?}", json.labels.len());

    // let parsed_edges = json.edges.into_iter()
    // .flat_map(|(key, edge)| {
    //     ParsedEdge::from_raw(key, RawEdge::Head(edge)).unwrap()
    // })
    // .collect::<Vec<_>>();



    // let mut scopes = json.data.into_keys()
    // .map(|s| ParsedScope::from_str(&s))
    // .collect::<ParseResult<Vec<_>>>()?;


    // let scope = scopes.get(165).unwrap();
    // let (scopes, relevant_edges) = get_scopegraph_section(scope, &scopes, &parsed_edges, 20, 0);

    // // let scope = scopes.get(150).unwrap();
    // // let (scopes, relevant_edges) = get_scopegraph_section(scope, &scopes, &parsed_edges, 5);

    // // let scopes = json.data.into_keys().take(25).collect::<Vec<_>>();
    // // let relevant_edges = parsed_edges.into_iter().filter(|e| {
    // //     scopes.contains(&e.from) || scopes.contains(&e.to)
    // // })
    // // .collect::<Vec<_>>();

    // let mut graph = PlantUmlDiagram::new("raw data");
    // for s in scopes {
    //     let item = PlantUmlItem::node(&s.name, &s.name, NodeType::Node);
    //     // let item = MermaidItem::node(&s.name, &s.name, ItemShape::Circle);
    //     graph.push(item);
    // }

    // for e in relevant_edges {
    //     let item = MermaidItem::edge(&e.from.name, &e.to.name, &e.label, EdgeType::Solid);
    //     let item = PlantUmlItem::edge(&e.from.name, &e.to.name, &e.label, EdgeDirection::Unspecified);
    //     graph.push(item);
    // }

    // graph.render_to_file("output/parsed_graph.puml")?;
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
    let adj_edges = edges.iter().filter(|e| &e.from == scope || &e.to == scope);
    for edge in adj_edges {
        // println!("edge: {0:?}", edge);
        let other_scope = if &edge.from == scope {
            scopes.iter().find(|s| s == &&edge.to)
        } else {
            scopes.iter().find(|s| s == &&edge.from)
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