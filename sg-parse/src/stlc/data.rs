use std::fmt::Display;

use super::StlcType;

#[derive(Hash, Default, Clone, Debug, PartialEq, Eq)]
pub enum StlcData {
    #[default]
    NoData,
    Variable(String, StlcType),
}

impl StlcData {
    pub fn datatype(&self) -> Option<&StlcType> {
        match self {
            StlcData::Variable(_, ty) => Some(ty),
            _ => None,
        }
    }
}

impl Display for StlcData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StlcData::NoData => write!(f, "NoData"),
            StlcData::Variable(name, ty) => write!(f, "{name}: {ty}"),
        }
    }
}
