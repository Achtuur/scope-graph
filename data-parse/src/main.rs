#![allow(unused)]
use std::{collections::{HashMap, HashSet}, fs::{File, OpenOptions}, io::BufReader};

mod raw;
mod parsed;
mod error;

pub use error::*;
use serde::Deserialize;

use crate::{parsed::{ParsedLabel, ParsedScope}, raw::{JavaType, JavaValue, RawLabel, RawQueryData, RawScope, RawScopeGraph, RefType}};

const BASE_PATH: &str = "./raw/";
const QUERIES_FILE: &str = "commons-csv.queries.json";
const RESULTS_FILE: &str = "commons-csv.results.json";
const SCOPEGRAPH_FILE: &str = "commons-csv.scopegraph.json";

fn main() -> ParseResult<()> {
    
    queries_data()?;
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

    let mut d = des.get_mut(3).unwrap();

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
    let mut json: serde_json::Value = Deserialize::deserialize(&mut deserializer)?;
    println!("{:?}", timer.elapsed());

    let map = json.as_object_mut().unwrap();
    // let mut arr = json.as_array().unwrap();

    println!("keys {0:?}", map.keys().collect::<Vec<_>>());

    let data_map = map.get("data").unwrap().as_object().unwrap();
    let data = data_map.iter().nth(50).unwrap();
    println!("data: {0:#?}", data);

    // let mut set = HashSet::new();
    // for (k, v) in data_map.iter() {
    //     let op = v.get("op").and_then(|op| op.as_str());
    //     if let Some(op) = op {
    //         set.insert(op);
    //     }
    // }
    // println!("set: {0:?}", set);

    let data_parsed: HashMap<String, JavaValue> = serde_json::from_value(map.remove("data").unwrap().clone()).unwrap();
    println!("data_parsed.len(): {0:?}", data_parsed.len());

    let actual_data = data_parsed.into_iter()
        .filter_map(|(k, v)| {
            if let JavaValue::Data(d) = v {
                Some((k, d))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    let scopes = actual_data.values()
    .filter_map(|v| ParsedScope::try_from(v.clone()).ok())
    .collect::<Vec<_>>();
    println!("scopes: {0:?}", scopes.len());

    let names = scopes.iter()
        .fold(HashSet::new(), |mut acc, s| {
            acc.insert(&s.name);
            acc
        });
    println!("names.len(): {0:?}", names.len());

    let d = actual_data.iter()
    .filter(|(_, v)| matches!(v, JavaType::Ref(_)))
    .nth(16);

    println!("d: {0:#?}", d);

    // let found = actual_data.values()
    // .filter_map(|v| {
    //     if let RawScopeGraphData::Scope(ref_data) = v {
    //         Some(ref_data)
    //     } else {
    //         None
    //     }
    // })
    // // .inspect(|s| println!("scope name: {0}", s.arg1.value))
    // .find(|s| s.arg1.value == "s_ty-1224");

    // println!("found: {0:?}", found);



    // let labels: Vec<RawLabel> = serde_json::from_value(map.remove("labels").unwrap()).unwrap();

    // let parsed = labels.into_iter()
    //     .map(ParsedLabel::from)
    //     .collect::<Vec<_>>();
    // println!("labels: {0:?}", parsed);



    // let labels = map.get("labels").unwrap().as_array().unwrap();
    // let lab = labels.iter().next().unwrap();
    // println!("lab: {0:#?}", lab);

    // let edges = map.get("edges").unwrap().as_object().unwrap();
    // let edge = edges.iter().next().unwrap();
    // println!("edge: {0:#?}", edge.1);


    // let json: RawScopeGraph = Deserialize::deserialize(&mut deserializer)?;
    // println!("json.data: {0:?}", json.data.len());
    Ok(())
}