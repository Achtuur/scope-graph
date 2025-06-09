use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct RawQueryData {
    pub dataOrd: serde_json::Value,
    pub dataWf: RawDataWf,
    pub labelOrd: serde_json::Value,
    pub pathWf: serde_json::Value,
    pub scope: serde_json::Value,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawDataWf {
    pub body: serde_json::Value,
    pub bodyCriticalEdges: serde_json::Value,
    pub freeVars: serde_json::Value,
    pub label: String,
    pub name: String,
    pub params: Vec<DataWfParams>,
}


#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DataWfParams {
    // empty tag is array of params
    Arr(ArrParams),
    Wildcard(WildcardWf),
    IdMatch(IdMatchWf),
}

impl DataWfParams {
    // arr with arg.len() == 1 should be flattened to the first element
    pub fn flatten_arrs(&mut self) {
        if let DataWfParams::Arr(ArrParams { args, common }) = self {
            match args.len() {
                0 => (),
                1 => *self = args.remove(0),
                _ => {
                    // if there are multiple args, flatten each of them
                    for arg in args.iter_mut() {
                        arg.flatten_arrs();
                    }
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct DataWfCommon {
    constructed: bool,
    op: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct ArrParams {
    args: Vec<DataWfParams>,
    #[serde(flatten)]
    common: DataWfCommon,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IdMatchWf {
    value: String,

    #[serde(flatten)]
    common: DataWfCommon,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WildcardWf {
    var: serde_json::Value,
    wildcard: bool,

    #[serde(flatten)]
    common: DataWfCommon,
}
