use serde::{Deserialize, Serialize};

use crate::raw::RawScope;

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
#[serde(tag = "op")]
pub enum JavaType {
    /// Direct scope declarations
    #[serde(rename = "Scope")]
    Scope(RawScope),

    #[serde(rename = "REF")]
    Ref(RefType),
    #[serde(rename = "")] // empty string is on purpose
    MethodOrClass(MethodOrClassData),
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
    MethodOrClass(MethodOrClassData),
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
    // #[serde(flatten)]
    // #[serde(skip_serializing)]
    // ignored: IgnoredFields,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawId {
    // arg0.value is the id
    pub arg0: ArgValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TypedData {
    arg0: serde_json::Value,
    #[serde(flatten)]
    #[serde(skip_serializing)]
    ignored: IgnoredFields,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ByteData {
    // #[serde(flatten)]
    // ignored: IgnoredFields,
    #[serde(flatten)]
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JavaRef<T> {
    // arg0 is the objec that is being referenced
    pub arg0: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MethodOrClassData {
    Method(MethodData),
    Class(ClassData),
}

impl MethodOrClassData {
    pub fn into_id_scope(self) -> (String, RawScope) {
        match self {
            MethodOrClassData::Method(data) => (data.arg0.arg0.value, data.arg2),
            MethodOrClassData::Class(data) => (data.arg0.arg0.value, data.arg1),
        }
    }

    pub fn id(&self) -> &str {
        match self {
            MethodOrClassData::Method(data) => &data.arg0.arg0.value,
            MethodOrClassData::Class(data) => &data.arg0.arg0.value,
        }
    }

    /// Scope that this method is defined in
    pub fn scope(&self) -> &RawScope {
        match self {
            MethodOrClassData::Method(data) => &data.arg2,
            MethodOrClassData::Class(data) => &data.arg1,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MethodData {
    arg0: RawId,
    arg2: RawScope,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClassData {
    arg0: RawId,
    arg1: RawScope,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_class_or_method() {
        let file = std::fs::File::open("./test_data/d-60398.json").unwrap();

        let parsed: JavaType = serde_json::from_reader(file).unwrap();
        println!("parsed: {0:?}", parsed);
    }

    #[test]
    fn test_parse_ref_type() {
        let file = std::fs::File::open("./test_data/d_50826-0.json").unwrap();

        let parsed: JavaType = serde_json::from_reader(file).unwrap();
        println!("parsed: {0:?}", parsed);
    }
}
