use scopegraphs::{
    completeness::ImplicitClose, label_order, query_regex, resolve::Resolve, Scope, ScopeGraph,
};

mod data;
mod label;
mod types;

pub use data::*;
pub use label::*;
pub use types::*;

pub type StlcGraph<'s> = ScopeGraph<'s, StlcLabel, StlcData, ImplicitClose<StlcLabel>>;

pub(crate) struct SgExpression<'a>(&'a sclang::SclangExpression);

impl<'a> SgExpression<'a> {
    pub fn new(expr: &'a sclang::SclangExpression) -> Self {
        SgExpression(expr)
    }

    // making this unsafe since i really quickly want a global counter, ill make it nice later i promise
    pub fn expr_type(&self, sg: &StlcGraph<'_>, prev_scope: Scope) -> StlcType {
        use sclang::SclangExpression as E;
        match &self.0 {
            E::Literal(_) => StlcType::Num,
            E::Boolean(_) => StlcType::Bool,
            E::Var(name) => {
                // query the scopegraph for the name of this thing and return the type
                let var_query = sg
                    .query()
                    .with_path_wellformedness(
                        query_regex!(StlcLabel: (Parent|Record|Extension)*Declaration),
                    )
                    .with_label_order(label_order!(StlcLabel:
                        Declaration < Parent,
                        Declaration < Record,
                        Declaration < Extension,
                        Record < Parent,
                        Record < Extension
                    ))
                    .with_data_wellformedness(|data: &StlcData| -> bool {
                        matches!(data, StlcData::Variable(d_name, _) if d_name == name)
                    })
                    .resolve(prev_scope);
                // println!("var_query: {:#?}", var_query);
                match var_query
                    .into_iter()
                    .nth(0)
                    .expect("Variable not found")
                    .data()
                {
                    StlcData::Variable(_, ty) => ty.clone(),
                    _ => panic!("Variable found but no type"),
                }
            }
            E::Add(lhs, rhs) => {
                // check if both expressions are numbers and then return a number
                let lhs = Self::new(lhs);
                let rhs = Self::new(rhs);
                let ty1 = lhs.expr_type(sg, prev_scope);
                let ty2 = rhs.expr_type(sg, prev_scope);
                if ty1 == StlcType::Num && ty2 == StlcType::Num {
                    StlcType::Num
                } else {
                    panic!("Addition of non-numbers")
                }
            }
            E::Func {
                param,
                p_type,
                body,
            } => {
                // add new scope for the function
                let new_scope = sg.add_scope_default();
                sg.add_edge(new_scope, StlcLabel::Parent, prev_scope)
                    .unwrap();

                // add scope for parameter declaration
                let t_param = StlcType::from_syntax_type(p_type, sg, prev_scope);
                let param_data = StlcData::Variable(param.to_string(), t_param.clone());
                sg.add_decl(new_scope, StlcLabel::Declaration, param_data)
                    .unwrap();

                // construct scopes for body using new scope
                let body = Self::new(body);
                let body_type = body.expr_type(sg, new_scope);
                StlcType::fun(t_param, body_type)
            }
            E::Call { fun, arg } => {
                let fun = Self::new(fun);
                let arg = Self::new(arg);
                let func_type = fun.expr_type(sg, prev_scope);

                let (t1, t2) = match func_type {
                    StlcType::Fun(t1, t2) => (t1, t2),
                    _ => panic!("Attempted to call non-function"),
                };

                let arg_type = arg.expr_type(sg, prev_scope);

                if !t1.is_subtype_of(&arg_type, sg) {
                    panic!("Parameter type mismatch")
                }
                *t2
            }
            E::Let { name, body, tail } => {
                let body = Self::new(body);
                let tail = Self::new(tail);
                // add new scope for the current "line"
                let new_scope = sg.add_scope_default();
                // let new_scope = sg.add_scope_default();
                sg.add_edge(new_scope, StlcLabel::Parent, prev_scope)
                    .unwrap();

                // add scope for var declaration
                let data = StlcData::Variable(name.to_string(), body.expr_type(sg, prev_scope));
                sg.add_decl(new_scope, StlcLabel::Declaration, data)
                    .unwrap();

                // construct scopes for body and tail using new_scope
                tail.expr_type(sg, new_scope)
            }
            E::Record(fields) => {
                // create a scope n that declares the record parameters
                let record_scope = sg.add_scope_default();
                // declare fields
                for (name, expr) in fields {
                    let expr = Self::new(expr);
                    let field_type = expr.expr_type(sg, prev_scope);
                    let field_data = StlcData::Variable(name.to_string(), field_type);
                    sg.add_decl(record_scope, StlcLabel::Declaration, field_data)
                        .unwrap();
                }

                StlcType::Record(record_scope.0)
            }
            E::RecordAccess { record, field } => {
                let record = Self::new(record);
                let record_type = record.expr_type(sg, prev_scope);
                let StlcType::Record(scope_num) = record_type else {
                    panic!("RecordAccess on non-record")
                };

                // query scope_num for field
                let query = sg.query()
                .with_path_wellformedness(query_regex!(StlcLabel: (Record|Extension)*Declaration)) // follow R or E edge until declaration
                .with_label_order(label_order!(StlcLabel: Record < Extension, Declaration < Record, Declaration < Extension)) // R < E, $ < R, $ < E
                .with_data_wellformedness(|data: &StlcData| -> bool {
                    matches!(data, StlcData::Variable(d_name, _) if d_name == field)
                })
                .resolve(Scope(scope_num));
                query
                    .get_only_item()
                    .expect("Field not found")
                    .data()
                    .datatype()
                    .expect("Data has no type")
                    .clone()
            }
            E::Extension {
                extension,
                parent: original,
            } => {
                let extension = Self::new(extension);
                let original = Self::new(original);
                let ext_scope = sg.add_scope_default();
                // extension must be record type
                let ext_t = extension.expr_type(sg, prev_scope);
                let StlcType::Record(ext_rec) = ext_t else {
                    panic!("Extension type is not record")
                };

                // original must be record type
                let orig_t = original.expr_type(sg, prev_scope);
                let StlcType::Record(r) = orig_t else {
                    panic!("Extending a non-record type")
                };

                // ext_scope -R> ext_rec
                // ext_scope -E> r
                sg.add_edge(ext_scope, StlcLabel::Record, Scope(ext_rec))
                    .unwrap();
                sg.add_edge(ext_scope, StlcLabel::Extension, Scope(r))
                    .unwrap();
                StlcType::Record(ext_scope.0)
            }
            E::With { record, body } => {
                let record = Self::new(record);
                let body = Self::new(body);
                let record_type = record.expr_type(sg, prev_scope);
                let StlcType::Record(r) = record_type else {
                    panic!("With on non-record")
                };

                let with_scope = sg.add_scope_default();
                sg.add_edge(with_scope, StlcLabel::Record, Scope(r))
                    .unwrap();
                sg.add_edge(with_scope, StlcLabel::Parent, prev_scope)
                    .unwrap();

                body.expr_type(sg, with_scope)
            }
        }
    }
}
