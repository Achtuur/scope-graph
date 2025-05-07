// tests from:
// https://github.com/metaborg/nabl/blob/master/statix.test/scopegraphs/nameresolution.spt

use scope_graph::{data::ScopeGraphData, graph::{CachedScopeGraph, ScopeGraph}, label::ScopeGraphLabel, order::{LabelOrder, LabelOrderBuilder}, regex::{dfs::RegexAutomata, Regex}, scope::Scope, DRAW_CACHES};
use serde::Serialize;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
enum TestLabel {
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
            TestLabel::P => 'P',
            TestLabel::Q => 'Q',
            TestLabel::R => 'R',
        }
    }

    fn str(&self) -> &'static str {
        match self {
            TestLabel::P => "P",
            TestLabel::Q => "Q",
            TestLabel::R => "R",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
enum TestData {
    NoData,
    Var(String),
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
        }
    }
}

impl TestData {
    fn var(name: impl ToString) -> Self {
        Self::Var(name.to_string())
    }

    fn name(&self) -> &str {
        match self {
            Self::NoData => "no data",
            Self::Var(name) => name,
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
    let mut graph = CachedScopeGraph::new();
    let s = graph.add_scope(Scope::new(), TestData::NoData);
    let regex = Regex::kleene(TestLabel::P).compile();
    let lo = LabelOrderBuilder::default().build();
    graph.query_proj(s,
        &regex,
        &lo,
        |_| (),
        (),
        |d1, d2| d1 == d2,
    );
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
    let mut graph = CachedScopeGraph::new();
    let s1 = graph.add_scope(Scope::new(), TestData::var("x"));
    let s2 = graph.add_scope(Scope::new(), TestData::var("x"));
    graph.add_edge(s1, s2, TestLabel::P);
    let regex = Regex::Character(TestLabel::P).compile();
    let lo = LabelOrderBuilder::default().build();
    let envs = graph.query_proj(s1,
        &regex,
        &lo,
        |d| d.name().to_string(),
        String::from("x"),
        |d1, d2| d1 == d2,
    );
    assert!(envs.len() == 1);
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
    let mut graph = CachedScopeGraph::new();
    let s1 = graph.add_scope(Scope::new(), TestData::NoData);
    let s2 = graph.add_scope(Scope::new(), TestData::var("x"));
    let s3 = graph.add_scope(Scope::new(), TestData::var("x"));
    graph.add_edge(s1, s2, TestLabel::P);
    graph.add_edge(s1, s3, TestLabel::Q);
    // this isnt supported
    // let regex = Regex::neg(Regex::ZeroSet).compile();
    let regex = Regex::kleene(
        Regex::or(TestLabel::P, TestLabel::Q)
    )
    .compile();

    let lo = LabelOrderBuilder::new()
    .push(TestLabel::Q, TestLabel::P)
    .build();
    let envs = graph.query_proj(s1,
        &regex,
        &lo,
        |d| d.name().to_string(),
        String::from("x"),
        |d1, d2| d1 == d2,
    );

    assert!(envs.len() == 1);
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
    let mut graph = CachedScopeGraph::new();
    let s1 = graph.add_scope(Scope::new(), TestData::NoData);
    let regex: RegexAutomata<TestLabel> = Regex::EmptyString.compile();

    let lo = LabelOrderBuilder::new().build();
    let envs = graph.query_proj(s1,
        &regex,
        &lo,
        |d| d.name().to_string(),
        String::from("x"),
        |d1, d2| d1 == d2,
    );

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
    let mut graph = CachedScopeGraph::new();
    let s1 = graph.add_scope(Scope::new(), TestData::NoData);
    let regex: RegexAutomata<TestLabel> = Regex::EmptyString.compile();

    let lo = LabelOrderBuilder::new()
    .push(TestLabel::P, TestLabel::Q)
    .build();
    let envs = graph.query_proj(s1,
        &regex,
        &lo,
        |_| (),
        (),
        |d1, d2| d1 == d2,
    );

    assert!(envs.is_empty());
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
    let mut graph = CachedScopeGraph::new();
    let s1 = graph.add_scope(Scope::new(), TestData::NoData);
    let regex: RegexAutomata<TestLabel> = Regex::EmptyString.compile();

    let lo = LabelOrderBuilder::new()
    .push(TestLabel::P, TestLabel::R)
    .push(TestLabel::Q, TestLabel::R)
    .build();
    let envs = graph.query_proj(s1,
        &regex,
        &lo,
        |_| (),
        (),
        |d1, d2| d1 == d2,
    );

    assert!(envs.is_empty());
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
    let mut graph = CachedScopeGraph::new();
    let s0 = graph.add_scope(Scope::new(), TestData::NoData);
    let s_with = graph.add_scope(Scope::new(), TestData::NoData);
    let s_rec = graph.add_scope(Scope::new(), TestData::var("x"));
    let s_let = graph.add_scope(Scope::new(), TestData::var("x"));
    graph.add_edge(s_with, s0, TestLabel::P);
    graph.add_edge(s_with, s_rec, TestLabel::R);
    graph.add_edge(s_let, s_with, TestLabel::P);
    graph.as_mmd_diagram("test_label_order_resp", DRAW_CACHES)
    .write_to_file("output/tests/test_label_order_resp.md").unwrap();
    let regex: RegexAutomata<TestLabel> = Regex::concat(
        Regex::kleene(TestLabel::P), Regex::question(TestLabel::R)
    )
    .compile();
    regex.to_mmd().write_to_file("output/tests/test_label_order_resp_regex.md").unwrap();

    let lo = LabelOrderBuilder::new()
    .push(TestLabel::R, TestLabel::P)
    .build();
    let envs = graph.query_proj(s_let,
        &regex,
        &lo,
        |d| d.name().to_string(),
        String::from("x"),
        |d1, d2| d1 == d2,
    );
    println!("envs: {0:?}", envs);
    assert!(envs.len() == 1);
    let env = envs.first().unwrap();
    assert!(env.data.name() == "x");
    assert!(env.path.target() == s_let);
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

#[test]
fn test_all_is_respected() {
    let mut graph = CachedScopeGraph::new();
    let s0 = graph.add_scope(Scope::new(), TestData::NoData);
    let s1 = graph.add_scope(Scope::new(), TestData::NoData);
    let s2 = graph.add_scope(Scope::new(), TestData::NoData);
    let s3 = graph.add_scope(Scope::new(), TestData::NoData);
    graph.add_edge(s0, s1, TestLabel::P);
    graph.add_edge(s0, s2, TestLabel::P);
    graph.add_edge(s1, s3, TestLabel::P);
    graph.add_edge(s2, s3, TestLabel::P);
    graph.as_mmd_diagram("test_all_is_respected", DRAW_CACHES)
    .write_to_file("output/tests/test_all_is_respected.md").unwrap();
    let regex: RegexAutomata<TestLabel> = Regex::kleene(TestLabel::P)
    .compile();
    regex.to_mmd().write_to_file("output/tests/test_all_is_respected_regex.md").unwrap();

    let lo = LabelOrderBuilder::new().build();
    let envs = graph.query_proj(s0,
        &regex,
        &lo,
        |d| d.name().to_string(),
        String::from("x"),
        |d1, d2| d1 == d2,
    );
    assert!(!envs.is_empty());
}

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
