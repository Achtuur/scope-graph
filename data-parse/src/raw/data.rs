use serde::Deserialize;

use crate::raw::{ArgValue, ConstructorArg, IgnoredFields};


#[derive(Deserialize, Debug, Clone)]
#[serde(tag="op", rename="Scope")]
pub struct RawScope {
    // /// arg1.value contains resource name,
    // /// to prevent duplicate names
    // pub arg0: ArgValue,
    // /// arg1.value contains scope name
    // pub arg1: ArgValue,
    /// [0].value is resource name (prevent duplicates)
    /// [1].value is scope name
    pub args: Vec<ArgValue>,

    // value: String,
    #[serde(flatten)]
    ignored: IgnoredFields,
    // #[serde(flatten)]
    // data: serde_json::Value,
}