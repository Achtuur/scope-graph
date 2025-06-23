use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{raw::{JavaRef, JavaType, RawEdge, RawEdgeHead, RawEdgeKey, RawEdgeTail, RawLabel, RawScope, RawScopeGraph, RefType}, ParseResult};

// https://stackoverflow.com/questions/51276896/how-do-i-use-serde-to-serialize-a-hashmap-with-structs-as-keys-to-json
pub mod vectorize {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::iter::FromIterator;

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        let container: Vec<_> = target.into_iter().collect();
        serde::Serialize::serialize(&container, ser)
    }

    pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        let container: Vec<_> = serde::Deserialize::deserialize(des)?;
        Ok(T::from_iter(container))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ScopeData {
    Ref(ParsedScope),
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedScopeGraph {
    #[serde(with = "vectorize")]
    pub scopes: HashMap<ParsedScope, ScopeData>,
    pub edges: Vec<ParsedEdge>,
    pub labels: Vec<ParsedLabel>,
}

impl TryFrom<RawScopeGraph> for ParsedScopeGraph {
    type Error = crate::ParseError;

    fn try_from(raw: RawScopeGraph) -> ParseResult<Self> {
        let mut scopes = raw.data.into_iter()
        .map(|(scope_key, data)| {
            let s = ParsedScope::from_str(&scope_key)?;
            let d = match data.into_data() {
                Some(JavaType::Scope(_)) => ScopeData::None, // scope declaration is not data, but good
                Some(JavaType::Ref(RefType::ScopeRef(raw_scope))) => {
                    ScopeData::Ref(ParsedScope::from(raw_scope.arg0))
                }
                Some(_) => ScopeData::None,
                None => ScopeData::None,
            };
            Ok((s, d))
        })
        .collect::<ParseResult<HashMap<_, _>>>()?;

        let edges = raw.edges.into_iter()
        .flat_map(|(key, edge)| {
            ParsedEdge::from_raw(key, RawEdge::Head(edge)).unwrap()
        })
        .collect::<Vec<_>>();

        let labels = raw.labels.into_iter().map(ParsedLabel::from)
        .collect::<Vec<_>>();

        Ok(Self {
            scopes,
            edges,
            labels,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParsedScope {
    pub resource: String,
    pub name: String,
}

impl FromStr for ParsedScope {
    type Err = crate::ParseError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let mut split = s.split("-");
        let resource = split.next().ok_or("Invalid scope format")?.trim_start_matches("#");
        let name = split.collect::<Vec<_>>().join("-");

        Ok(Self {
            resource: resource.to_string(),
            name,
        })
    }
}

impl From<RawScope> for ParsedScope {
    fn from(raw: RawScope) -> Self {
        ParsedScope {
            // resource: raw.args[0].value.clone(),
            // name: raw.args[1].value.clone(),
            resource: raw.resource,
            name: raw.name,
        }
    }
}

impl ParsedScope {
    pub fn id(&self) -> String {
        format!("#/./{}-{}", self.resource, self.name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParsedEdge {
    pub from: ParsedScope,
    pub to: ParsedScope,
    pub label: String,
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

        let edges = to.into_iter()
            .map(|h| {
                let to = ParsedScope::from(h);
                ParsedEdge {
                    from: from.clone(),
                    to,
                    label: key.label.clone(),
                }
            })
            .collect();
        Ok(edges)
    }
}