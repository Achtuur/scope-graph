use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::RawLabel;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub enum JavaLabel {
    // "java/JRE!typeName"
    TypeName,
    // "java/names/Main!TYPE_PARAMS"
    TypeParams,
    // "java/names/Main!IMPORT_CU"
    ImportCu,
    // "java/names/Main!STATIC_MEMBERS"
    StaticMember,
    // "java/names/MethodNames!return"
    Return,
    // "java/names/PackageNames!thisPkg"
    LocalPackage,
    // "java/names/MethodNames!mthd"
    Method,
    // "java/names/TypeNames!thisType"
    LocalType,
    // "java/types/Main!withKind"
    WithKind,
    // "java/names/PackageNames!pkg"
    Package,
    // "java/names/TypeNames!type"
    JType,
    // "java/names/Main!STATIC_LEX"
    StaticParent,
    // "java/names/ExpressionNames!var"
    VarDecl,
    // "java/names/Main!IMPLEMENTS"
    Impl,
    // "java/types/Conversions!box"
    Boxed,
    // "java/names/Main!EXTENDS"
    Extend,
    // "java/types/Main!withType"
    WithType,
    // "java/names/Main!IMPORT_PKG"
    ImportPackage,
    // "java/names/Main!STATIC_IMPORT_ONDEMAND"
    ImportStaticOndemand,
    // "java/names/Main!SINGLE_TYPE_IMPORT"
    ImportSingleType,
    // "java/names/Main!LEX"
    Parent,
    // "java/names/Main!PARENT_PKG"
    ParentPackage,
    // "java/names/Main!TYPE_IMPORT_ONDEMAND"
    ImportTypeOndemand,
    // "java/names/Main!SINGLE_STATIC_IMPORT"
    ImportSingleStatic,
    // "java/types/ReferenceTypes!elementType"
    ElementType,
}

impl<'a> TryFrom<&'a str> for JavaLabel {
    type Error = crate::ParseError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value.trim_start_matches("\\").trim_end_matches("\\") {
            "java/JRE!typeName" => Ok(Self::TypeName),
            "java/names/Main!TYPE_PARAMS" => Ok(Self::TypeParams),
            "java/names/Main!IMPORT_CU" => Ok(Self::ImportCu),
            "java/names/Main!STATIC_MEMBERS" => Ok(Self::StaticMember),
            "java/names/MethodNames!return" => Ok(Self::Return),
            "java/names/PackageNames!thisPkg" => Ok(Self::LocalPackage),
            "java/names/MethodNames!mthd" => Ok(Self::Method),
            "java/names/TypeNames!thisType" => Ok(Self::LocalType),
            "java/types/Main!withKind" => Ok(Self::WithKind),
            "java/names/PackageNames!pkg" => Ok(Self::Package),
            "java/names/TypeNames!type" => Ok(Self::JType),
            "java/names/Main!STATIC_LEX" => Ok(Self::StaticParent),
            "java/names/ExpressionNames!var" => Ok(Self::VarDecl),
            "java/names/Main!IMPLEMENTS" => Ok(Self::Impl),
            "java/types/Conversions!box" => Ok(Self::Boxed),
            "java/names/Main!EXTENDS" => Ok(Self::Extend),
            "java/types/Main!withType" => Ok(Self::WithType),
            "java/names/Main!IMPORT_PKG" => Ok(Self::ImportPackage),
            "java/names/Main!STATIC_IMPORT_ONDEMAND" => Ok(Self::ImportStaticOndemand),
            "java/names/Main!SINGLE_TYPE_IMPORT" => Ok(Self::ImportSingleType),
            "java/names/Main!LEX" => Ok(Self::Parent),
            "java/names/Main!PARENT_PKG" => Ok(Self::ParentPackage),
            "java/names/Main!TYPE_IMPORT_ONDEMAND" => Ok(Self::ImportTypeOndemand),
            "java/names/Main!SINGLE_STATIC_IMPORT" => Ok(Self::ImportSingleStatic),
            "java/types/ReferenceTypes!elementType" => Ok(Self::ElementType),

            _ => {
                println!("Found unknown label: {}", value);
                Err("Unknown label".into())
            }
        }
    }
}

impl TryFrom<String> for JavaLabel {
    type Error = crate::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        JavaLabel::try_from(value.as_str())
    }
}

impl TryFrom<ParsedLabel> for JavaLabel {
    type Error = crate::ParseError;

    fn try_from(value: ParsedLabel) -> Result<Self, Self::Error> {
        JavaLabel::try_from(value.name.as_str())
    }
}

impl Display for JavaLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl JavaLabel {
    pub fn cosmo_color(&self) -> &'static str {
        match self {
            Self::Extend | Self::Impl => "#fa98e3",
            Self::StaticParent | Self::Parent => "#fabbeb",

            Self::StaticMember | Self::Boxed | Self::VarDecl => "#c0fab9",

            Self::ImportStaticOndemand
            | Self::ImportSingleType
            | Self::ImportTypeOndemand
            | Self::ImportSingleStatic
            | Self::ImportCu
            | Self::ImportPackage => "#ebe8ebff",

            Self::Method | Self::Return => "#ebe9e8ff",

            Self::LocalPackage | Self::ParentPackage | Self::Package => "#ebe8e8ff",
            Self::TypeName
            | Self::LocalType
            | Self::ElementType
            | Self::TypeParams
            | Self::WithKind
            | Self::WithType
            | Self::JType => "#e8ebeaff",
        }
    }

    pub fn cosmo_value(&self) -> usize {
        match self {
            JavaLabel::VarDecl => 20,
            JavaLabel::Parent => 10,
            JavaLabel::Extend => 5,
            JavaLabel::Impl => 4,
            JavaLabel::Return => 20,
            _ => 1,
        }
    }

    pub fn cosmo_width(&self) -> usize {
        1
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
