use serde::Deserialize;

use crate::raw::{JavaRef, JavaType, RawLabel, RawScope, RefType};

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[serde(from = "RawScope")]
pub struct ParsedScope {
    pub name: String,
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