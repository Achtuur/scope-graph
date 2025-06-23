use serde::{Deserialize, Serialize};

use crate::raw::{RawScope};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IgnoredFields {
    #[serde(default)]
    isGround: i32,
    #[serde(default)]
    hashCode: i32,
    #[serde(default)]
    ground: bool,
    #[serde(default)]
    arity: i32,
    #[serde(default)]
    args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JavaValue {
    /// Variants with an "op" field
    Data(JavaType),
    /// 4 out of 100.000 variants don't have an "op" field
    SomeBullshit(serde_json::Value),
}

impl JavaValue {
    pub fn into_data(self) -> Option<JavaType> {
        match self {
            JavaValue::Data(data) => Some(data),
            JavaValue::SomeBullshit(_) => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag="op")]
pub enum JavaType {
    /// Direct scope declarations
    #[serde(rename = "Scope")]
    Scope(RawScope),

    #[serde(rename = "REF")]
    Ref(RefType),
    #[serde(other)]
    Unknown,

    // #[serde(rename = "BYTE")]
    // Byte(ByteData),
    // #[serde(rename = "ARRAY")]
    // Array(serde_json::Value),
    // #[serde(rename = "VOID")]
    // Void(serde_json::Value),
    // #[serde(rename = "TYPED")]
    // Typed(serde_json::Value),
    // #[serde(rename = "SHORT")]
    // Short(serde_json::Value),
    // #[serde(rename = "INTF")]
    // Inft(serde_json::Value),
    // #[serde(rename = "DOUBLE")]
    // Double(serde_json::Value),
    // #[serde(rename = "CLASS")]
    // Class(serde_json::Value),
    // #[serde(rename = "FLOAT")]
    // Float(serde_json::Value),
    // #[serde(rename = "BOOLEAN")]
    // Boolean(serde_json::Value),
    // #[serde(rename = "AMBTYPE")]
    // AmbType(serde_json::Value),
    // #[serde(rename = "CHAR")]
    // Char(serde_json::Value),
    // #[serde(rename = "LONG")]
    // Long(serde_json::Value),
}

/// Enum that represents all values that are used with a REF tag
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RefType {
    /// Ref to a scope object? I guess this creates a new scope?
    ScopeRef(JavaRef<RawScope>),
    #[serde(skip_serializing)]
    Ref(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConstructorArg {
    Value(ArgValue),
    Object(Box<JavaType>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArgValue {
    pub value: String,
    #[serde(flatten)]
    #[serde(skip_serializing)]
    ignored: IgnoredFields,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag="op", rename="TYPED")]
pub struct TypedData {
    arg0: serde_json::Value,
    #[serde(flatten)]
    #[serde(skip_serializing)]
    ignored: IgnoredFields,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "BYTE", tag="op")]
pub struct ByteData {
    // #[serde(flatten)]
    // ignored: IgnoredFields,
    #[serde(flatten)]
    data: serde_json::Value,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag="op", rename="REF")]
pub struct JavaRef<T>
{
    // arg0 is the objec that is being referenced
    pub arg0: T
}