// tests from:
// https://github.com/metaborg/nabl/blob/master/statix.test/scopegraphs/nameresolution.spt

use std::rc::Rc;

use deepsize::DeepSizeOf;
use graphing::Renderer;
use scope_graph::{
    DRAW_CACHES,
    data::ScopeGraphData,
    graph::{CachedScopeGraph, ScopeGraph},
    label::ScopeGraphLabel,
    order::LabelOrderBuilder,
    projection::ScopeGraphDataProjection,
    regex::{Regex, dfs::RegexAutomaton},
    scope::Scope,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord, DeepSizeOf)]
enum TestLabel {
    D,
    P,
    Q,
    R,
}

impl std::fmt::Display for TestLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl ScopeGraphLabel for TestLabel {
    fn char(&self) -> char {
        match self {
            TestLabel::D => '$',
            TestLabel::P => 'P',
            TestLabel::Q => 'Q',
            TestLabel::R => 'R',
        }
    }

    fn str(&self) -> &'static str {
        match self {
            TestLabel::D => "$",
            TestLabel::P => "P",
            TestLabel::Q => "Q",
            TestLabel::R => "R",
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord, DeepSizeOf)]
enum TestData {
    #[default]
    NoData,
    Var(String),
    VarNum(String, usize),
}

impl std::fmt::Display for TestData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render_string())
    }
}

impl ScopeGraphData for TestData {
    fn variant_has_data(&self) -> bool {
        matches!(self, Self::Var(_))
    }

    fn render_string(&self) -> String {
        match self {
            Self::NoData => String::new(),
            Self::Var(name) => name.clone(),
            Self::VarNum(name, num) => format!("{name}{num}"),
        }
    }

    fn render_with_type(&self) -> String {
        self.render_string()
    }
}

impl TestData {
    fn var(name: impl ToString) -> Self {
        Self::Var(name.to_string())
    }

    fn varnum(name: impl ToString, num: usize) -> Self {
        Self::VarNum(name.to_string(), num)
    }

    fn name(&self) -> &str {
        match self {
            Self::NoData => "no data",
            Self::Var(name) => name,
            Self::VarNum(name, _) => name,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum TestProjection {
    None,
    Name,
}

impl ScopeGraphDataProjection<TestData> for TestProjection {
    type Output = String;

    fn project(&self, data: &TestData) -> Self::Output {
        match self {
            TestProjection::None => String::new(),
            TestProjection::Name => data.name().to_string(),
        }
    }
}

/// ```ignore
/// test query no-data succeeds [[
///   resolve {s}
///     query () filter P* in s |-> _
///   signature
///     name-resolution
///       labels
///         P
/// ]] analysis succeeds
/// ```
#[test]
fn test_no_data() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope_default();
    let regex = Regex::kleene(TestLabel::P).compile();
    let lo = LabelOrderBuilder::default().build();
    graph.query_proj(s, &regex, &lo, (), ());
}

// test namespace resolve with labels wf succeeds [[
//   resolve true
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var filter P*
// ]] analysis succeeds

// test namespace resolve with relation wf fails [[
//   resolve true
//   signature
//     relations
//       r : int
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var filter P* r
// ]] analysis fails

// test namespace resolve with labels ord succeeds [[
//   resolve true
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P Q
//       resolve Var min P < Q
// ]] analysis succeeds

// test namespace resolve with eop placeholder ord succeeds [[
//   resolve true
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels Q
//       resolve Var min $ < Q
// ]] analysis succeeds

// test namespace resolve with relation ord fails [[
//   resolve true
//   signature
//     relations
//       r : int
//     namespaces
//       Var : string
//     name-resolution
//       labels Q
//       resolve Var min r < Q
// ]] analysis fails

// test resolve reference with same name in the same scope succeeds [[
//   resolve {s}
//     new s, s -> Var{"x"@-},
//     Var{"x"@-} in s |-> [_]
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_resolve_reference_with_same_name() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope(Scope::new(), TestData::var("x"));

    let regex = Regex::EmptyString.compile();
    let lo = LabelOrderBuilder::default().build();
    let envs = graph.query_proj(s, &regex, &lo, TestProjection::Name, String::from("x"));
    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(env.path.target() == s);
}

// test resolve reference with different name in the same scope fails [[
//   resolve {s}
//     new s, s -> Var{"x"@-},
//     Var{"y"@-} in s |-> [_]
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var
// ]] analysis succeeds
//    run evaluate-test to FAILS()

#[test]
fn test_resolve_reference_with_different_name() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope(Scope::new(), TestData::var("y"));

    let regex = Regex::EmptyString.compile();
    let lo = LabelOrderBuilder::default().build();
    let envs = graph.query_proj(s, &regex, &lo, TestProjection::Name, String::from("x"));
    assert!(envs.is_empty());
}

// test resolution policy filter forces a step [[
//   resolve {s1 s2}
//     new s1 s2, s1 -P-> s2,
//     s1 -> Var{"x"@s1},
//     s2 -> Var{"x"@s2},
//     Var{"x"@-} in s1 |-> [(_, Var{_@s2})]
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var filter P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_resolution_policy_forces_step() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s1 = graph.add_scope(Scope::new(), TestData::var("x"));
    let s2 = graph.add_scope(Scope::new(), TestData::var("x"));
    graph.add_edge(s1, s2, TestLabel::P);

    let regex = Regex::from(TestLabel::P).compile();
    let lo = LabelOrderBuilder::default().build();
    let envs = graph.query_proj(s1, &regex, &lo, TestProjection::Name, String::from("x"));
    println!("envs;: {0:?}", envs);
    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(env.path.target() == s2);
}

/// No edge in graph but an env is still found, even though it shouldnt
#[test]
fn test_no_edge_has_env() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s1 = graph.add_scope(Scope::new(), TestData::var("x"));
    let _s2 = graph.add_scope(Scope::new(), TestData::var("x"));

    let regex = Regex::from(TestLabel::P).compile();
    let lo = LabelOrderBuilder::default().build();
    let envs = graph.query_proj(s1, &regex, &lo, TestProjection::Name, String::from("x"));
    assert!(envs.is_empty());
}

// test resolution policy filter cannot reach declaration [[
//   resolve {s1 s2}
//     new s1 s2, s1 -P-> s2,
//     s1 -> Var{"x"@s1},
//     Var{"x"@-} in s1 |-> []
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var filter P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_resolution_policy_filter_cannot_reach() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s1 = graph.add_scope(Scope::new(), TestData::var("x"));
    let s2 = graph.add_scope(Scope::new(), TestData::NoData);
    graph.add_edge(s1, s2, TestLabel::P);

    let regex = Regex::from(TestLabel::P).compile();
    let lo = LabelOrderBuilder::default().build();
    let envs = graph.query_proj(s1, &regex, &lo, TestProjection::Name, String::from("x"));
    assert!(envs.is_empty());
}

// test resolution policy min is applied [[
//   resolve {s1 s2 s3}
//     new s1 s2 s3,
//     s1 -P-> s2, s2 -> Var{"x"@s2},
//     s1 -Q-> s3, s3 -> Var{"x"@s3},
//     Var{"x"@-} in s1 |-> [(_, Var{_@s3})]
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P Q
//       resolve Var min Q < P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_resolution_policy_min_is_applied() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s1 = graph.add_scope(Scope::new(), TestData::NoData);
    let s2 = graph.add_scope(Scope::new(), TestData::var("x"));
    let s3 = graph.add_scope(Scope::new(), TestData::var("x"));

    graph.add_edge(s1, s2, TestLabel::P);
    graph.add_edge(s1, s3, TestLabel::Q);

    graph
        .as_mmd_diagram("test_resolution_policy_min_is_applied", DRAW_CACHES)
        .render_to_file("output/tests/test_resolution_policy_min_is_applied.md")
        .unwrap();

    // let regex = Regex::EmptyString.compile();
    let regex = Regex::or(TestLabel::P, TestLabel::Q).compile();
    let lo = LabelOrderBuilder::default()
        .push(TestLabel::Q, TestLabel::P)
        .build();
    let envs = graph.query_proj(s1, &regex, &lo, TestProjection::Name, String::from("x"));
    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(env.data.name() == "x");
    assert!(env.path.target() == s3);
}

// test refer explicitly to resolution policy filter succeeds [[
//   resolve {s1 s2}
//     new s1 s2, s1 -P-> s2,
//     s1 -> Var{"x"@s1},
//     s2 -> Var{"x"@s2},
//     query decl filter resolveMatch[Var{_@-}] in s1 |-> [(_, Var{_@s2})]
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P Q
//       resolve Var filter P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()
#[test]
fn test_explicit_policy_filter() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s1 = graph.add_scope(Scope::new(), TestData::var("x"));
    let s2 = graph.add_scope(Scope::new(), TestData::var("x"));
    graph.add_edge(s1, s2, TestLabel::P);
    let regex = Regex::from(TestLabel::P).compile();
    let lo = LabelOrderBuilder::default().build();
    let envs = graph.query_proj(s1, &regex, &lo, TestProjection::Name, String::from("x"));
    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(env.data.name() == "x");
    assert!(env.path.target() == s2);
}

// test refer explicitly to resolution policy min succeeds [[
//   resolve {s1 s2 s3}
//     new s1 s2 s3,
//     s1 -P-> s2, s2 -> Var{"x"@s2},
//     s1 -Q-> s3, s3 -> Var{"x"@s3},
//     query decl filter ~0 min resolveLt[Var{_@-}] and true in s1 |-> [(_, Var{_@s3})]
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P Q
//       resolve Var min Q < P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_explicit_policy_min() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope(Scope::new(), TestData::var("x"));
    let s3 = graph.add_scope(Scope::new(), TestData::var("x"));
    graph.add_edge(s1, s2, TestLabel::P);
    graph.add_edge(s1, s3, TestLabel::Q);
    // this isnt supported
    // let regex = Regex::neg(Regex::ZeroSet).compile();
    let regex = Regex::kleene(Regex::or(TestLabel::P, TestLabel::Q)).compile();

    let lo = LabelOrderBuilder::new()
        .push(TestLabel::Q, TestLabel::P)
        .build();
    let envs = graph.query_proj(s1, &regex, &lo, TestProjection::Name, String::from("x"));

    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(env.data.name() == "x");
    assert!(env.path.target() == s3);
}

// test resolve occurrence relations in the same scope succeeds [[
//   resolve {s}
//     new s, !r[Var{"x"@-}, 1] in s,
//     r of Var{"x"@-} in s |-> [(_, (_, 1))]
//   signature
//     relations
//       r : occurrence -> int
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_resolve_occurence_relations_in_same_scope() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope_default();
    let _ = graph.add_decl(s, TestLabel::D, TestData::varnum("x", 1));

    let regex = Regex::from(TestLabel::D).compile();
    let lo = LabelOrderBuilder::new().build();
    let envs = graph.query_proj(s, &regex, &lo, TestProjection::Name, String::from("x"));

    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(matches!(*env.data, TestData::VarNum(_, 1)));
}

// test resolve occurrence relations with resolution policy succeeds [[
//   resolve {s1 s2 s3 s4}
//     new s1 s2 s3 s4,
//     s1 -P-> s2,
//     s2 -P-> s3, !r[Var{"x"@-}, 8] in s3,
//     s2 -Q-> s4, !r[Var{"x"@-}, 4] in s4,
//     r of Var{"x"@-} in s1 |-> [(_, (_, 4))]
//   signature
//     relations
//       r : occurrence -> int
//     namespaces
//       Var : string
//     name-resolution
//       labels P Q
//       resolve Var filter P (P|Q)* min Q < P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_resolve_occurence_relations_with_resolution() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope_default();
    let s3 = graph.add_scope_default();
    let s4 = graph.add_scope_default();
    graph.add_edge(s1, s2, TestLabel::P);
    graph.add_edge(s2, s3, TestLabel::P);
    graph.add_edge(s2, s4, TestLabel::Q);
    let _ = graph.add_decl(s3, TestLabel::D, TestData::varnum("x", 8));
    let _ = graph.add_decl(s4, TestLabel::D, TestData::varnum("x", 4));

    let regex = Regex::concat(
        Regex::concat(
            TestLabel::P,
            Regex::kleene(Regex::or(TestLabel::P, TestLabel::Q)),
        ),
        TestLabel::D,
    )
    .compile();
    let lo = LabelOrderBuilder::new()
        .push(TestLabel::Q, TestLabel::P)
        .build();
    let envs = graph.query_proj(s1, &regex, &lo, TestProjection::Name, String::from("x"));

    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(matches!(*env.data, TestData::VarNum(_, 4)));
}

// test relations have multiset behavior [[
//   resolve {s x y}
//     new s,
//     !r[Var{"x"@-}] in s,
//     !r[Var{"x"@-}] in s,
//     r of Var{"x"@-} in s |-> [_, _]
//   signature
//     relations
//       r : occurrence
//     namespaces
//       Var : string
//     name-resolution
//       resolve Var
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_relations_have_multiset_behavior() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope_default();
    let _ = graph.add_decl(s, TestLabel::D, TestData::var("x"));
    let _ = graph.add_decl(s, TestLabel::D, TestData::var("x"));

    graph
        .as_mmd_diagram("test_relations_have_multiset_behaviour", false)
        .render_to_file("output/tests/test_relations_have_multiset_behaviour.md")
        .unwrap();

    let regex = Regex::from(TestLabel::D).compile();
    let lo = LabelOrderBuilder::new().build();
    let envs = graph.query_proj(s, &regex, &lo, TestProjection::Name, String::from("x"));

    println!("envs: {0:?}", envs);

    assert_eq!(envs.len(), 2);
}

// test resolve declaration added using occurrence short-hand notation succeeds [[
//   resolve {s}
//     new s, s -> Var{"x"@-},
//     Var{"x"@-} in s |-> [(_, _)]
//   signature
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var filter P* min $ < P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_resolve_declaration_using_shorthand() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope_default();
    let _ = graph.add_decl(s, TestLabel::D, TestData::var("x"));

    let regex = Regex::concat(Regex::kleene(TestLabel::P), TestLabel::D).compile();
    let lo = LabelOrderBuilder::new()
        .push(TestLabel::D, TestLabel::P)
        .build();
    let envs = graph.query_proj(s, &regex, &lo, TestProjection::Name, String::from("x"));

    assert_eq!(envs.len(), 1);
}

// test resolve declaration added using occurrence + relation short-hand notation succeeds [[
//   resolve {s}
//     new s, s -> Var{"x"@-} with r 8,
//     Var{"x"@-} in s |-> [(_, _)]
//   signature
//     relations
//       r : occurrence -> int
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var filter P* min $ < P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

// test query relation added using occurrence + relation short-hand notation succeeds [[
//   resolve {s}
//     new s, s -> Var{"x"@-} with r 8,
//     r of Var{"x"@-} in s |-> [(_, (_, 8))]
//   signature
//     relations
//       r : occurrence -> int
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var filter P* min $ < P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_resolve_declaration_using_shorthand_with_relation_query() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope_default();
    let _ = graph.add_decl(s, TestLabel::D, TestData::varnum("x", 8));

    let regex = Regex::concat(Regex::kleene(TestLabel::P), TestLabel::D).compile();
    let lo = LabelOrderBuilder::new()
        .push(TestLabel::D, TestLabel::P)
        .build();
    let envs = graph.query_proj(s, &regex, &lo, TestProjection::Name, String::from("x"));

    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(matches!(*env.data, TestData::VarNum(_, 8)));
}

// test query relation added using occurrence + multiple relations short-hand notation succeeds [[
//   resolve {s}
//     new s, s -> Var{"x"@-} with r 8 and q "five",
//     r of Var{"x"@-} in s |-> [(_, (_, 8))],
//     q of Var{"x"@-} in s |-> [(_, (_, "five"))]
//   signature
//     relations
//       r : occurrence -> int
//       q : occurrence -> string
//     namespaces
//       Var : string
//     name-resolution
//       labels P
//       resolve Var filter P* min $ < P
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

// test partial order is well-behaved (1) [[
//   resolve {s}
//     new s,
//     !r[] in s,
//     query r min $ < P, $ < Q in s |-> [_]
//   signature
//     name-resolution
//       labels P Q
//     relations
//       r :
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_partial_order() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s1 = graph.add_scope_default();
    let regex: RegexAutomaton<TestLabel> = Regex::EmptyString.compile();

    let lo = LabelOrderBuilder::new().build();
    let envs = graph.query_proj(s1, &regex, &lo, TestProjection::Name, String::from("x"));

    assert!(envs.is_empty());
}

// test partial order is well-behaved (2) [[
//   resolve {s}
//     new s,
//     !r[] in s,
//     query r min $ < P, $ < Q, P < Q in s |-> [_]
//   signature
//     name-resolution
//       labels P Q
//     relations
//       r :
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_partial_order_2() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope_default();
    let regex: RegexAutomaton<TestLabel> = Regex::EmptyString.compile();

    let lo = LabelOrderBuilder::new()
        .push(TestLabel::D, TestLabel::P)
        .push(TestLabel::D, TestLabel::Q)
        .push(TestLabel::P, TestLabel::Q)
        .build();
    let envs = graph.query_proj(s, &regex, &lo, (), ());
    assert_eq!(envs.len(), 1);
    let first = envs.first().unwrap();
    assert!(first.data == Rc::from(TestData::NoData));
    assert!(first.path.target() == s);
}

// test partial order is well-behaved (3) [[
//   resolve {s}
//     new s,
//     !r[] in s,
//     query r min $ < P, $ < Q, P < R, Q < R in s |-> [_]
//   signature
//     name-resolution
//       labels P Q R
//     relations
//       r :
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_partial_order_3() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s = graph.add_scope_default();
    let regex: RegexAutomaton<TestLabel> = Regex::EmptyString.compile();

    let lo = LabelOrderBuilder::new()
        .push(TestLabel::D, TestLabel::P)
        .push(TestLabel::D, TestLabel::Q)
        .push(TestLabel::P, TestLabel::R)
        .push(TestLabel::Q, TestLabel::R)
        .build();
    let envs = graph.query_proj(s, &regex, &lo, (), ());

    assert_eq!(envs.len(), 1);
    let first = envs.first().unwrap();
    assert!(first.data == Rc::from(TestData::NoData));
    assert!(first.path.target() == s);
}

// test label order is respected [[
//   resolve {s0 s_with s_rec s_let}
//     new s0,
//     new s_with,
//         s_with -P-> s0,
//         s_with -R-> s_rec,
//     new s_rec,
//         !typeOfDecl["x", 1] in s_rec,
//     new s_let,
//         s_let -P-> s_with,
//         !typeOfDecl["x", 2] in s_let,
//     query typeOfDecl
//           filter P* R? and { "x" }
//           min $ < P, $ < R, R < P and true
//           in s_let |-> [(_, (_, 2))]
//   signature
//     namespaces
//       Var  : string
//     name-resolution
//       labels P R
//   relations
//       typeOfDecl : string -> int
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_label_order_respected() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s0 = graph.add_scope_default();
    let s_with = graph.add_scope_default();
    let s_rec = graph.add_scope_default();
    let s_let = graph.add_scope_default();
    let s_let_decl = graph.add_decl(s_let, TestLabel::D, TestData::var("x"));
    let _s_with_decl = graph.add_decl(s_rec, TestLabel::D, TestData::var("x"));
    graph.add_edge(s_with, s0, TestLabel::P);
    graph.add_edge(s_with, s_rec, TestLabel::R);
    graph.add_edge(s_let, s_with, TestLabel::P);
    graph
        .as_mmd_diagram("test_label_order_resp", DRAW_CACHES)
        .render_to_file("output/tests/test_label_order_resp.md")
        .unwrap();
    let regex: RegexAutomaton<TestLabel> = Regex::concat(
        Regex::concat(Regex::kleene(TestLabel::P), Regex::question(TestLabel::R)),
        TestLabel::D,
    )
    .compile();
    regex
        .to_mmd()
        .render_to_file("output/tests/test_label_order_resp_regex.md")
        .unwrap();

    let lo = LabelOrderBuilder::new()
        .push(TestLabel::D, TestLabel::P)
        .push(TestLabel::D, TestLabel::R)
        .push(TestLabel::R, TestLabel::P)
        .build();
    let envs = graph.query_proj(s_let, &regex, &lo, TestProjection::Name, String::from("x"));
    println!("envs: {0:?}", envs);
    assert_eq!(envs.len(), 1);
    let env = envs.first().unwrap();
    assert!(env.data.name() == "x");
    assert!(env.path.target() == s_let_decl);
}

// test project all is respected [[
//   resolve {s0 s1 s2 s3}
//     new s0 s1 s2 s3,
//     s0 -P-> s1,
//     s0 -P-> s2,
//     s1 -P-> s3,
//     s2 -P-> s3,
//     query r
//           filter P* and { "x" }
//           project *
//           in s0 |-> _ : list((path * (string * int)))
//   signature
//     name-resolution
//       labels P
//   relations
//       r : string -> int
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

// #[test]
// fn test_all_is_respected() {
//     let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
//     let s0 = graph.add_scope_default();
//     let s1 = graph.add_scope_default();
//     let s2 = graph.add_scope_default();
//     let s3 = graph.add_scope_default();
//     graph.add_edge(s0, s1, TestLabel::P);
//     graph.add_edge(s0, s2, TestLabel::P);
//     graph.add_edge(s1, s3, TestLabel::P);
//     graph.add_edge(s2, s3, TestLabel::P);
//     graph.as_mmd_diagram("test_all_is_respected", DRAW_CACHES)
//     .render_to_file("output/tests/test_all_is_respected.md").unwrap();
//     let regex: RegexAutomaton<TestLabel> = Regex::kleene(TestLabel::P)
//     .compile();
//     regex.to_mmd().render_to_file("output/tests/test_all_is_respected_regex.md").unwrap();

//     let lo = LabelOrderBuilder::new().build();
//     let envs = graph.query_proj(s0,
//         &regex,
//         &lo,
//         TestProjection::Name,
//         String::from("x"),
//
//     );
//     assert!(!envs.is_empty());
// }

// test project target and data is respected [[
//   resolve {s0 s1 s2 s3}
//     new s0 s1 s2 s3,
//     s0 -P-> s1,
//     s0 -P-> s2,
//     s1 -P-> s3,
//     s2 -P-> s3,
//     query r
//           filter P* and { "x" }
//           project dst, $
//           in s0 |-> _ : list((scope * (string * int)))
//   signature
//     name-resolution
//       labels P
//   relations
//       r : string -> int
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

// test project target and data behaves as set [[
//   resolve {s0 s1 s2 s3}
//     new s0 s1 s2 s3,
//     s0 -P-> s1,
//     s0 -P-> s2,
//     s1 -P-> s3,
//     s2 -P-> s3,
//     !r[1] in s3,
//     query r
//           filter P*
//           project dst, $
//           in s0 |-> [(_, 1)]
//   signature
//     name-resolution
//       labels P
//   relations
//       r : int
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

#[test]
fn test_project_target_data_behaves_as_set() {
    let mut graph = CachedScopeGraph::<TestLabel, TestData>::new();
    let s0 = graph.add_scope_default();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope_default();
    let s3 = graph.add_scope_default();
    let s3_decl = graph.add_decl(s3, TestLabel::D, TestData::var("x"));
    graph.add_edge(s0, s1, TestLabel::P);
    graph.add_edge(s0, s2, TestLabel::P);
    graph.add_edge(s1, s3, TestLabel::P);
    graph.add_edge(s2, s3, TestLabel::P);
    graph
        .as_mmd_diagram("test_project_target_data_behaves_as_set", DRAW_CACHES)
        .render_to_file("output/tests/test_project_target_data_behaves_as_set.md")
        .unwrap();
    let regex: RegexAutomaton<TestLabel> =
        Regex::concat(Regex::kleene(TestLabel::P), TestLabel::D).compile();
    regex
        .to_mmd()
        .render_to_file("output/tests/test_label_order_resp_regex.md")
        .unwrap();

    let lo = LabelOrderBuilder::new()
        .push(TestLabel::D, TestLabel::P)
        .build();
    let envs = graph.query_proj(s0, &regex, &lo, TestProjection::Name, String::from("x"));
    println!("envs: {0:?}", envs);
    assert!(!envs.is_empty());
    for env in envs {
        assert!(env.data.name() == "x");
        assert!(env.path.target() == s3_decl);
    }
}

// test project data is respected [[
//   resolve {s0 s1 s2 s3}
//     new s0 s1 s2 s3,
//     s0 -P-> s1,
//     s0 -P-> s2,
//     s1 -P-> s3,
//     s2 -P-> s3,
//     query r
//           filter P* and { "x" }
//           project $
//           in s0 |-> _ : list((string * int))
//   signature
//     name-resolution
//       labels P
//   relations
//       r : string -> int
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()

// test project data behaves as set [[
//   resolve {s0 s1 s2 s3}
//     new s0 s1 s2 s3,
//     s0 -P-> s1,
//     s0 -P-> s2,
//     s1 -P-> s3,
//     s2 -P-> s3,
//     !r[1] in s3,
//     query r
//           filter P*
//           project $
//           in s0 |-> [1]
//   signature
//     name-resolution
//       labels P
//   relations
//       r : int
// ]] analysis succeeds
//    run evaluate-test to SUCCEEDS()
