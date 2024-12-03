use std::fmt::Display;

use sclang::SclangType;
use scopegraphs::{label_order, query_regex, resolve::Resolve, Scope};

use crate::StlcLabel;

use super::{StlcData, StlcGraph};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntaxTypes {
    Num,
    Bool,
    Fun(Box<SyntaxTypes>, Box<SyntaxTypes>),
    Record(Vec<(String, SyntaxTypes)>),
}

impl SyntaxTypes {
    pub fn to_stlc(&self, sg: &StlcGraph<'_>, prev_scope: Scope) -> StlcType {
        match self {
            Self::Num => StlcType::Num,
            Self::Bool => StlcType::Bool,
            Self::Fun(t1, t2) => {
                let t1 = t1.to_stlc(sg, prev_scope);
                let t2 = t2.to_stlc(sg, prev_scope);
                StlcType::Fun(Box::new(t1), Box::new(t2))
            }
            Self::Record(fields) => {
                // create a scope n that declares the record parameters
                let record_scope = sg.add_scope_default();
                // declare fields
                for (name, expr) in fields {
                    let field_type = expr.to_stlc(sg, prev_scope);
                    let field_data = StlcData::Variable(name.to_string(), field_type);
                    sg.add_decl(record_scope, StlcLabel::Declaration, field_data)
                        .unwrap();
                }
                StlcType::Record(record_scope.0)
            }
        }
    }
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum StlcType {
    Num,
    Bool,
    Fun(Box<StlcType>, Box<StlcType>),
    // number is the scope number
    Record(usize),
}

// constructor utility
impl StlcType {
    pub fn fun(param_type: StlcType, return_type: StlcType) -> StlcType {
        StlcType::Fun(Box::new(param_type), Box::new(return_type))
    }
}

impl StlcType {
    pub fn from_syntax_type(s_t: &SclangType, sg: &StlcGraph<'_>, prev_scope: Scope) -> Self {
        match s_t {
            SclangType::Num => StlcType::Num,
            SclangType::Bool => StlcType::Bool,
            SclangType::Fun(t1, t2) => {
                let t1 = Self::from_syntax_type(t1, sg, prev_scope);
                let t2 = Self::from_syntax_type(t2, sg, prev_scope);
                StlcType::Fun(Box::new(t1), Box::new(t2))
            }
            SclangType::Record(fields) => {
                // create a scope n that declares the record parameters
                let record_scope = sg.add_scope_default();
                // declare fields
                for (name, expr) in fields {
                    let field_type = Self::from_syntax_type(expr, sg, prev_scope);
                    let field_data = StlcData::Variable(name.to_string(), field_type);
                    sg.add_decl(record_scope, StlcLabel::Declaration, field_data)
                        .unwrap();
                }
                StlcType::Record(record_scope.0)
            }
        }
    }

    pub fn is_subtype_of(&self, other: &Self, sg: &StlcGraph<'_>) -> bool {
        match (self, other) {
            // trivial
            (Self::Num, Self::Num) => true,
            (Self::Bool, Self::Bool) => true,

            (Self::Fun(t1, t2), Self::Fun(u1, u2)) => {
                t1.is_subtype_of(u1, sg) && t2.is_subtype_of(u2, sg)
            }
            (Self::Record(r1), Self::Record(r2)) => {
                let query = sg.query()
                .with_path_wellformedness(query_regex!(StlcLabel: (Record|Extension)*Declaration)) // follow R or E edge until declaration
                .with_label_order(label_order!(StlcLabel: Record < Extension, Declaration < Record, Declaration < Extension)) // R < E, $ < R, $ < E
                .with_data_wellformedness(|data: &StlcData| -> bool {
                    matches!(data, StlcData::Variable(_, _))
                });

                // r1 is subtype of r2 is all of r1's types are subtypes of r2's types
                let res_sub = query.resolve(Scope(*r1));
                let res_super = query.resolve(Scope(*r2));

                let is_subtype = res_sub.iter().all(|sub| {
                    let StlcData::Variable(x_sub, t_sub) = sub.data() else {
                        panic!("Record subtype query somehow returned non-variable");
                    };
                    res_super.iter().any(|sup| {
                        let StlcData::Variable(x_super, t_super) = sup.data() else {
                            panic!("Record subtype query somehow returned non-variable");
                        };
                        x_sub == x_super
                        && t_sub.is_subtype_of(t_super, sg)
                    })
                });
                is_subtype
            }
            (t1, t2) if t1 != t2 => false,
            _ => unimplemented!("Subtyping for {:?} and {:?}", self, other),
        }
    }
}

impl Display for StlcType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StlcType::Num => write!(f, "num"),
            StlcType::Bool => write!(f, "bool"),
            StlcType::Fun(param_type, return_type) => write!(f, "({param_type} -> {return_type})"),
            StlcType::Record(n) => write!(f, "REC({n})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtype() {
        let t1 = StlcType::Record(1);
        let t2 = StlcType::Record(2);
    }
}
