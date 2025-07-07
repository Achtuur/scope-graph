use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::raw::{ArgValue, IgnoredFields, JavaValue};

#[derive(Serialize, Deserialize, Debug)]
pub struct RawScopeGraph {
    // key is scope name, value is data
    pub data: HashMap<String, JavaValue>,
    pub labels: Vec<RawLabel>,
    pub edges: HashMap<RawEdgeKey, RawEdgeHead>,
}

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct RawEdgeKey {
    pub s1: String,
    pub label: String,
}

impl TryFrom<String> for RawEdgeKey {
    type Error = crate::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // input looks like: "<#/./jdk8-s_ty-4564, Label(\"java/names/Main!EXTENDS\")>"
        // which should be parsed as:
        // - scope: "#/./jdk8-s_ty-4564"
        // - label: "java/names/Main!EXTENDS"

        let mut parts = value
            .trim_start_matches("<")
            .trim_end_matches(">")
            .split(",");

        let scope_name = parts
            .next()
            .map(|s| s.trim())
            .ok_or("Invalid edge format")?;
        let label_name = parts
            .next()
            .map(|s| {
                s.trim()
                    .trim_start_matches("Label(\"")
                    .trim_end_matches("\")")
            })
            .ok_or("Invalid edge format")?;
        Ok(Self {
            s1: scope_name.to_string(),
            label: label_name.to_string(),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "op", rename = "Label")]
pub struct RawLabel {
    /// arg0.value contains scope name
    pub arg0: ArgValue,
    #[serde(flatten)]
    #[serde(skip_serializing)]
    ignored: IgnoredFields,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RawEdge {
    Head(RawEdgeHead),
    Tail(RawEdgeTail),
}

impl RawEdge {
    pub fn split(self) -> (Option<RawScope>, Option<RawEdgeTail>) {
        match self {
            RawEdge::Head(e) => (Some(e.head), Some(*e.tail)),
            RawEdge::Tail(e) => (e.head, e.tail.map(|t| *t)),
        }
    }

    pub fn head(&self) -> Option<&RawScope> {
        match self {
            RawEdge::Head(head) => Some(&head.head),
            RawEdge::Tail(tail) => tail.head.as_ref(),
        }
    }

    pub fn tail(&self) -> Option<&RawEdgeTail> {
        match self {
            RawEdge::Head(head) => Some(&head.tail),
            RawEdge::Tail(tail) => Some(tail),
        }
    }
}

/// head/tail is a linked list, convert to a vec by just taking all unique values
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "op", rename = "Edge")]
pub struct RawEdgeHead {
    /// Head always has a head
    pub head: RawScope,
    // there's always a tail with nullable fields
    pub tail: Box<RawEdgeTail>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "op", rename = "Edge")]
pub struct RawEdgeTail {
    pub head: Option<RawScope>,
    pub tail: Option<Box<RawEdgeTail>>,
}

impl RawEdgeTail {
    pub fn depth(&self) -> usize {
        match &self.tail {
            Some(tail) => 1 + tail.depth(),
            None => 1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RawScope {
    /// Sometimes there's directly a name and resource field available
    Direct { name: String, resource: String },
    /// ...and sometimes it's in "arg" format
    InArgs {
        // arg0 is resource
        arg0: ArgValue,
        // arg1 is scope name
        arg1: ArgValue,
    },
}

impl RawScope {
    /// Returns (name, resource) pair
    pub fn into_name_resource(self) -> (String, String) {
        match self {
            Self::Direct { name, resource } => (name, resource),
            Self::InArgs { arg0, arg1 } => (arg1.value, arg0.value),
        }
    }
}
