use std::{hash::Hash, str::FromStr};

use graphing::plantuml::NodeType;
use serde::{Deserialize, Serialize};

use crate::{
    JavaLabel, ParseResult,
    raw::{RawEdge, RawEdgeKey, RawScope},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParsedScope {
    pub resource: String,
    pub name: String,
}

impl FromStr for ParsedScope {
    type Err = crate::ParseError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let mut split = s.split("-");
        let resource = split
            .next()
            .ok_or("Invalid scope format")?
            .trim_start_matches("#");
        let name = split.collect::<Vec<_>>().join("-");

        Ok(Self {
            resource: resource.to_string(),
            name,
        })
    }
}

impl From<RawScope> for ParsedScope {
    fn from(raw: RawScope) -> Self {
        let (name, resource) = raw.into_name_resource();
        ParsedScope { resource, name }
    }
}

impl ParsedScope {
    pub fn new(name: impl Into<String>, resource: impl Into<String>) -> Self {
        ParsedScope {
            resource: resource.into(),
            name: name.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> String {
        format!("{}-{}", self.resource, self.name)
    }

    pub fn cosmo_color(&self) -> &'static str {
        match self {
            s if s.is_class() => "#e8e8eb",
            s if s.is_data() => "#c0fab9",
            s if s.is_method() => "#fa98e3",
            s if s.is_method_body() => "#fabbeb",
            s if s.is_var() => "#94fc88",
            _ => "#f07070"
        }
    }

    pub fn is_data(&self) -> bool {
        self.name.starts_with("d-") || self.name.starts_with("d_")
    }

    pub fn is_method(&self) -> bool {
        self.name.contains("_mthd_")
    }

    pub fn is_method_body(&self) -> bool {
        self.name.contains("_mthdHead") || self.name.contains("_mthdBody")
    }

    pub fn is_class(&self) -> bool {
        self.name.contains("type") || self.name.contains("_ty-")
    }

    pub fn is_var(&self) -> bool {
        self.name.contains("var")
    }

    pub fn graph_node_type(&self) -> NodeType {
        match self.is_data() {
            true => NodeType::Card,
            false => NodeType::Node,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ParsedEdge {
    pub from: ParsedScope,
    pub to: ParsedScope,
    pub label: JavaLabel,
}

impl ParsedEdge {
    pub fn from_raw(key: RawEdgeKey, head: RawEdge) -> ParseResult<Vec<Self>> {
        let mut to = Vec::new();
        let mut cur_head = head;

        while let (Some(h), Some(t)) = cur_head.split() {
            to.push(h);
            cur_head = RawEdge::Tail(t);
        }

        let from = ParsedScope::from_str(&key.s1)?;

        let edges = to
            .into_iter()
            .map(|h| {
                let to = ParsedScope::from(h);
                let label = JavaLabel::try_from(key.label.as_str())?;
                Ok(ParsedEdge {
                    from: from.clone(),
                    to,
                    label,
                })
            })
            .collect::<ParseResult<Vec<_>>>()?;
        Ok(edges)
    }
}
