use std::{collections::HashMap, rc::Rc};

use serde::{de::DeserializeOwned, Deserialize};

mod data;
mod java;
mod query;

pub use data::*;
pub use java::*;
pub use query::*;

#[derive(Deserialize, Debug)]
pub struct RawScopeGraph {
    pub data: HashMap<String, JavaType>,
    pub labels: Vec<RawLabel>,
    pub edges: HashMap<String, RawEdge>,
}


#[derive(Deserialize, Debug)]
#[serde(tag="op", rename="Label")]
pub struct RawLabel {
    /// arg0.value contains scope name
    pub arg0: ArgValue,
    #[serde(flatten)]
    ignored: IgnoredFields,
}

#[derive(Deserialize, Debug)]
#[serde(tag="op", rename="Edge")]
pub struct RawEdge {
    head: RawScope,
    tail: Option<Box<RawEdge>>,
    #[serde(flatten)]
    ignored: IgnoredFields,
}