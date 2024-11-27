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

/// Expressions that can appear in the SLTC language
///
/// Extended with records
#[derive(Debug)]
pub enum Expression {
    Literal(i32),
    Boolean(bool),
    Var(String),
    // arg1, arg2
    Add(Box<Expression>, Box<Expression>),
    // parameter, parameter type, body
    Func {
        param_name: String,
        param_type: SyntaxTypes,
        body: Box<Expression>,
    },
    Call {
        func: Box<Expression>,
        argument: Box<Expression>,
    },
    Let {
        name: String,
        body: Box<Expression>,
        tail: Box<Expression>,
    },
    Record(Vec<(String, Expression)>),

    RecordAccess {
        record: Box<Expression>,
        field: String,
    },

    Extension {
        /// Added/overwritten fields
        extension: Box<Expression>,
        /// Parent record
        parent: Box<Expression>,
    },

    With {
        record: Box<Expression>,
        body: Box<Expression>,
    },
}

// utility constructors
impl Expression {
    fn var(name: impl ToString) -> Self {
        Expression::Var(name.to_string())
    }
    // just some utility
    fn add(lhs: Expression, rhs: Expression) -> Self {
        Expression::Add(Box::new(lhs), Box::new(rhs))
    }

    fn func(param_name: impl ToString, param_type: SyntaxTypes, body: Expression) -> Self {
        Expression::Func {
            param_name: param_name.to_string(),
            param_type,
            body: Box::new(body),
        }
    }

    fn call(func: Expression, param: Expression) -> Self {
        Expression::Call {
            func: Box::new(func),
            argument: Box::new(param),
        }
    }

    fn let_expr(name: impl ToString, body: Expression, tail: Expression) -> Self {
        Expression::Let {
            name: name.to_string(),
            body: Box::new(body),
            tail: Box::new(tail),
        }
    }

    fn rec(fields: Vec<(impl ToString, Expression)>) -> Self {
        let f = fields
            .into_iter()
            .map(|(name, expr)| (name.to_string(), expr))
            .collect();
        Expression::Record(f)
    }

    fn rec_access(record: impl ToString, field: impl ToString) -> Self {
        Expression::RecordAccess {
            record: Box::new(Expression::Var(record.to_string())),
            field: field.to_string(),
        }
    }

    fn extend(extension: Expression, parent: Expression) -> Self {
        Expression::Extension {
            extension: Box::new(extension),
            parent: Box::new(parent),
        }
    }

    fn with(record: Expression, body: Expression) -> Self {
        Expression::With {
            record: Box::new(record),
            body: Box::new(body),
        }
    }
}

impl Expression {
    /*
       let x = false in
       let x = 3 in
       let y = x in
       let f = fun(x: num) { let z = x in true } in
       f y
    */
    /// returns an example program, equivalent to the one in the paper
    pub fn example_program() -> Self {
        Expression::let_expr(
            "x",
            Expression::Boolean(false),
            Expression::let_expr(
                "x",
                Expression::Literal(3),
                Expression::let_expr(
                    "y",
                    Expression::Var("x".to_string()),
                    Expression::let_expr(
                        "f",
                        Expression::func(
                            "x",
                            SyntaxTypes::Num,
                            Expression::let_expr(
                                "z",
                                Expression::var("x"),
                                Expression::Boolean(false),
                            ),
                        ), // x and z are unused
                        Expression::call(Expression::var("f"), Expression::var("y")),
                    ),
                ),
            ),
        )
    }

    pub fn example_program_rec() -> Self {
        // let r = {x = 3, y = false} in
        // let a = r.x in
        // let q = {z = 4, x = true} extends r in
        // let b = q.x in
        // let p = with q
        // do
        //     let z = z in
        //     z
        // in
        // a

        let rec_expr = Expression::Record(vec![
            (String::from("x"), Expression::Literal(3)),
            (String::from("y"), Expression::Boolean(false)),
        ]);
        let ext_rec = Expression::Record(vec![
            (String::from("z"), Expression::Literal(4)),
            (String::from("x"), Expression::Boolean(true)),
        ]);
        let with_statement = Expression::with(
            Expression::var("q"),
            Expression::let_expr("z", Expression::var("z"), Expression::var("z")),
        );
        Expression::let_expr(
            "r",
            rec_expr,
            Expression::let_expr(
                "a",
                Expression::rec_access("r", "x"),
                Expression::let_expr(
                    "q",
                    Expression::extend(ext_rec, Expression::var("r")),
                    Expression::let_expr(
                        "b",
                        Expression::rec_access("q", "x"),
                        Expression::let_expr("p", with_statement, Expression::var("a")),
                    ),
                ),
            ),
        )
    }

    pub fn example_program_subtyping() -> Self {
        // let r = {x = 3, y = false, z = 5} in
        // let f = fun(arg: {x: num, y: bool}) { arg.x } in
        // let a = f r in
        // a

        let fn_expr = Expression::func(
            "arg",
            SyntaxTypes::Record(vec![
                ("x".to_string(), SyntaxTypes::Num),
                ("y".to_string(), SyntaxTypes::Bool),
            ]),
            Expression::rec_access("arg", "x"),
        );

        Expression::let_expr(
            "r",
            Expression::rec(vec![
                ("x", Expression::Literal(3)),
                ("y", Expression::Boolean(false)),
                ("z", Expression::Literal(5)),
            ]),
            Expression::let_expr(
                "f",
                fn_expr,
                Expression::let_expr(
                    "a",
                    Expression::call(Expression::var("f"), Expression::var("r")),
                    Expression::var("a"),
                ),
            ),
        )
    }

    // making this unsafe since i really quickly want a global counter, ill make it nice later i promise
    pub fn expr_type(&self, sg: &StlcGraph<'_>, prev_scope: Scope) -> StlcType {
        match self {
            Expression::Literal(_) => StlcType::Num,
            Expression::Boolean(_) => StlcType::Bool,
            Expression::Var(name) => {
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
            Expression::Add(lhs, rhs) => {
                // check if both expressions are numbers and then return a number
                let ty1 = lhs.expr_type(sg, prev_scope);
                let ty2 = rhs.expr_type(sg, prev_scope);
                if ty1 == StlcType::Num && ty2 == StlcType::Num {
                    StlcType::Num
                } else {
                    panic!("Addition of non-numbers")
                }
            }
            Expression::Func {
                param_name,
                param_type,
                body,
            } => {
                // add new scope for the function
                let new_scope = sg.add_scope_default();
                sg.add_edge(new_scope, StlcLabel::Parent, prev_scope)
                    .unwrap();

                // add scope for parameter declaration
                let t_param = param_type.to_stlc(sg, prev_scope);
                let param_data = StlcData::Variable(param_name.to_string(), t_param.clone());
                sg.add_decl(new_scope, StlcLabel::Declaration, param_data)
                    .unwrap();

                // construct scopes for body using new scope
                let body_type = body.expr_type(sg, new_scope);
                StlcType::fun(t_param, body_type)
            }
            Expression::Call { func, argument } => {
                let func_type = func.expr_type(sg, prev_scope);

                let (t1, t2) = match func_type {
                    StlcType::Fun(t1, t2) => (t1, t2),
                    _ => panic!("Attempted to call non-function"),
                };

                let arg_type = argument.expr_type(sg, prev_scope);

                if !t1.is_subtype_of(&arg_type, sg) {
                    panic!("Parameter type mismatch")
                }
                *t2
            }
            Expression::Let { name, body, tail } => {
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
            Expression::Record(fields) => {
                // create a scope n that declares the record parameters
                let record_scope = sg.add_scope_default();
                // declare fields
                for (name, expr) in fields {
                    let field_type = expr.expr_type(sg, prev_scope);
                    let field_data = StlcData::Variable(name.to_string(), field_type);
                    sg.add_decl(record_scope, StlcLabel::Declaration, field_data)
                        .unwrap();
                }

                StlcType::Record(record_scope.0)
            }
            Expression::RecordAccess { record, field } => {
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
            Expression::Extension {
                extension,
                parent: original,
            } => {
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
            Expression::With { record, body } => {
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
