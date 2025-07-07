use std::fmt::Display;

use scope_graph::{data::ScopeGraphData, projection::ScopeGraphDataProjection};

use super::StlcType;

#[derive(Hash, Default, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
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

    pub fn name(&self) -> String {
        match self {
            StlcData::NoData => String::new(),
            StlcData::Variable(name, _) => name.to_string(),
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

impl ScopeGraphData for StlcData {
    fn variant_has_data(&self) -> bool {
        match self {
            Self::NoData => false,
            _ => true,
        }
    }

    fn render_string(&self) -> String {
        match self {
            StlcData::NoData => String::new(),
            StlcData::Variable(name, ty) => format!("{name}: {ty}"),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum StlcProjection {
    VarName,
    IsVar,
}

impl ScopeGraphDataProjection<StlcData> for StlcProjection {
    type Output = String;

    fn project(&self, data: &StlcData) -> Self::Output
    where
        StlcData: ScopeGraphData,
    {
        match self {
            StlcProjection::VarName => data.name(),
            StlcProjection::IsVar => match data {
                StlcData::Variable(_, _) => String::from("yes"),
                _ => String::from("no"),
            },
        }
    }
}
