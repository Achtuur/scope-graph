use serde::Deserialize;

use crate::raw::{JavaRef, JavaType, RawEdge, RawEdgeHead, RawEdgeKey, RawEdgeTail, RawLabel, RawScope, RefType};

#[derive(Deserialize, Debug, Clone)]
#[serde(from = "RawLabel")]
pub struct ParsedLabel {
    pub name: String,
}

impl From<RawLabel> for ParsedLabel {
    fn from(raw: RawLabel) -> Self {
        ParsedLabel {
            name: raw.arg0.value,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(from = "RawScope")]
pub struct ParsedScope {
    pub name: String,
}

impl ParsedScope {
    pub fn new(raw: &str) -> Self {
        Self {
            name: Self::parse_name(raw),
        }
    }

    pub fn parse_name(raw: &str) -> String {
        raw.split("-").skip(1).collect::<Vec<_>>().join("-")
    }
}

impl From<RawScope> for ParsedScope {
    fn from(raw: RawScope) -> Self {
        ParsedScope {
            name: raw.args[1].value.clone(),
        }
    }
}

impl From<JavaRef<RawScope>> for ParsedScope {
    fn from(raw: JavaRef<RawScope>) -> Self {
        raw.arg0.into()
    }
}

impl TryFrom<JavaType> for ParsedScope {
    type Error = crate::ParseError;

    fn try_from(raw: JavaType) -> core::result::Result<Self, Self::Error> {
        match raw {
            JavaType::Scope(s) => Ok(s.into()),
            JavaType::Ref(RefType::ScopeRef(s)) => Ok(s.into()),
            _ => Err("Invalid javatype for scope".into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

impl ParsedEdge {
    pub fn from_raw(key: RawEdgeKey, head: RawEdge) -> Vec<Self> {
        let mut to = Vec::new();
        let mut cur_head = head;

        while let (Some(h), Some(t)) = cur_head.split() {
            to.push(h);
            cur_head = RawEdge::Tail(t);
        }

        let from = ParsedScope::parse_name(&key.s1);

        to.into_iter()
            .map(|h| ParsedEdge {
                from: from.to_string(),
                to: h.name,
                label: key.label.clone(),
            })
            .collect()
    }
}

impl From<(RawEdgeKey, RawEdgeHead)> for ParsedEdge {
    fn from(raw: (RawEdgeKey, RawEdgeHead)) -> Self {
        todo!()
    }
}