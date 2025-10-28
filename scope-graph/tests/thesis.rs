
use graphing::Renderer;

use scope_graph::{bench_util::bench::PatternGenerator, generator::{GraphGenerator, GraphPattern}, graph::{CachedScopeGraph, GraphRenderOptions, LabelRenderStyle, ScopeGraph}, scope::Scope, SgData, SgLabel};

pub fn render_options() -> GraphRenderOptions {
    GraphRenderOptions {
        draw_caches: false,
        draw_labels: LabelRenderStyle::Long,
        draw_types: true,
        draw_node_label: true,
        draw_colors: true,
    }
}

pub fn table_render_options() -> GraphRenderOptions {
    GraphRenderOptions {
        draw_caches: false,
        draw_labels: LabelRenderStyle::None,
        draw_types: false,
        draw_node_label: false,
        draw_colors: false,
    }
}

type Graph = CachedScopeGraph<SgLabel, SgData>;

/*
int a = 3;
String b = "hello world";
class Stack {}
Stack s = new Stack();
*/
#[test]
fn thesis_name_decl() {
    let mut graph = Graph::new();
    let s0 = graph.add_scope_default();
    let s1 = graph.add_scope_default();
    let a_decl = graph.add_decl(s1, SgLabel::Declaration, SgData::var("a", "int"));
    let s2 = graph.add_scope_default();
    let b_decl = graph.add_decl(s2, SgLabel::Declaration, SgData::var("b", "String"));
    let s3 = graph.add_scope_default();
    let stack_decl = graph.add_decl(s3, SgLabel::Declaration, SgData::var("Stack", "Class"));
    let s4 = graph.add_scope_default();
    let stack_inst = graph.add_decl(s4, SgLabel::Declaration, SgData::var("s", "Stack"));
    graph.add_edge(s1, s0, SgLabel::Parent);
    graph.add_edge(s2, s1, SgLabel::Parent);
    graph.add_edge(s3, s2, SgLabel::Parent);
    graph.add_edge(s4, s3, SgLabel::Parent);
    graph.as_uml_diagram("NameDecl Example", &render_options())
        .render_to_file("output/thesis_name_decl.puml")
        .unwrap();
}
/*
int double(int x) { /* ... */ }
int x = 3;
double(x);
*/
#[test]
fn thesis_func_decl() {
    let mut graph = Graph::new();
    let s0 = graph.add_scope_default();
    let s1 = graph.add_scope_default();
    let s_f = graph.add_scope_default();
    graph.add_edge(s1, s0, SgLabel::Parent);
    graph.add_edge(s_f, s0, SgLabel::Parent);
    graph.add_decl(s_f, SgLabel::Declaration, SgData::var("x", "int"));
    graph.add_decl(s1, SgLabel::Declaration, SgData::var("f", "int -> int"));

    let s2 = graph.add_scope_default();
    graph.add_decl(s2, SgLabel::Declaration, SgData::var("x", "int"));
    graph.add_edge(s2, s1, SgLabel::Parent);


    // let func_decl = graph.add_decl(s0, SgLabel::Declaration, SgData::var("double", "int -> int"));
    // let s1 = graph.add_scope_default();
    // let x_decl = graph.add_decl(s1, SgLabel::Declaration, SgData::var("x", "int"));
    graph.as_uml_diagram("FuncDecl Example", &render_options())
        .render_to_file("output/thesis_func_decl.puml")
        .unwrap();
}

/*
class Stack {
    List inner;
    Stack() { /* ... */ }
    int pop() { /* ... */ }
}
Stack s = new Stack();
*/
#[test]
fn thesis_class() {
    let mut graph = Graph::new();
    let s0 = graph.add_scope_default();
    let s_stack = graph.add_decl(s0, SgLabel::Declaration, SgData::var("Stack", "Class"));
    let s2 = graph.add_scope_default();
    graph.add_edge(s2, s0, SgLabel::Parent);
    graph.add_decl(s_stack, SgLabel::Method, SgData::var("Stack", "Constructor"));
    graph.add_decl(s_stack, SgLabel::Method, SgData::var("pop", "() -> int"));
    graph.add_decl(s_stack, SgLabel::Declaration, SgData::var("inner", "List"));
    let mthd_body = graph.add_scope_default();
    graph.add_edge(mthd_body, s_stack, SgLabel::Parent);

    graph.add_decl(s2, SgLabel::Declaration, SgData::var("s", "Stack"));
    graph.as_uml_diagram("Class Example", &render_options())
        .render_to_file("output/thesis_class_decl.puml")
        .unwrap();
}
/*
class NonEmptyStack extends Stack {
    NonEmptyStack(int first) { /* ... */ }
    @Override int pop() { /* ... */ }
}
*/
#[test]
fn thesis_class_ext() {
    let mut graph = Graph::new();
    let s0 = graph.add_scope_default();
    let s_stack = graph.add_decl(s0, SgLabel::Declaration, SgData::var("Stack", "Class"));
    graph.add_decl(s_stack, SgLabel::Method, SgData::var("pop", "() -> int"));

    let s_ext = graph.add_decl(s0, SgLabel::Declaration, SgData::var("NonEmptyStack", "Class"));
    graph.add_edge(s_ext, s_stack, SgLabel::Extend);
    // graph.add_decl(s_ext, SgLabel::Method, SgData::var("NonEmptyStack", "Constructor"));
    graph.add_decl(s_ext, SgLabel::Method, SgData::var("pop", "() -> int"));
    graph.as_uml_diagram("Class Ext Example", &render_options())
        .render_to_file("output/thesis_class_ext_decl.puml")
        .unwrap();
}

/*
interface Clearable {
    void clear();
}

class Stack implements Clearable {
    List inner;
    Stack() { /* ... */ }
    int pop() { /* ... */ }
    // interface's methods
    int clear() { /* ... */}
}
*/
#[test]
fn thesis_interface() {
    let mut graph = Graph::new();
    let s0 = graph.add_scope_default();
    let s_stack = graph.add_decl(s0, SgLabel::Declaration, SgData::var("Stack", "Class"));
    graph.add_decl(s_stack, SgLabel::Method, SgData::var("clear", "() -> ()"));

    let s_interface = graph.add_decl(s0, SgLabel::Declaration, SgData::var("Clearable", "Interface"));
    graph.add_edge(s_stack, s_interface, SgLabel::Implement);
    // graph.add_decl(s_ext, SgLabel::Method, SgData::var("NonEmptyStack", "Constructor"));
    graph.add_decl(s_interface, SgLabel::Method, SgData::var("clear", "() -> ()"));
    graph.as_uml_diagram("Interface Example", &render_options())
        .render_to_file("output/thesis_interface.puml")
        .unwrap();
}
/*
interface Iterable {
    Iterator iterator();
}

interface Countable extends Iterable {
    int count();
}

interface Summable extends Iterable {
    int sum();
}

class Stack implements Countable, Summable {
    List inner;
    Stack() { /* ... */ }
    int pop() { /* ... */ }
    Iterator iterator() { /* ... */ }
    int count() { /* ... */ }
    int sum() { /* ... */ }
}
*/
#[test]
fn thesis_interface_diamond() {
    let mut graph = Graph::new();
    let s0 = graph.add_scope_default();
    let s_stack = graph.add_decl(s0, SgLabel::Declaration, SgData::var("Stack", "Class"));
    let s_iterable = graph.add_decl(s0, SgLabel::Declaration, SgData::var("Iterable", "Interface"));
    let s_countable = graph.add_decl(s0, SgLabel::Declaration, SgData::var("Countable", "Interface"));
    let s_summable = graph.add_decl(s0, SgLabel::Declaration, SgData::var("Summable", "Interface"));
    // graph.add_decl(s_stack, SgLabel::Method, SgData::var("iterator", "() -> Iterator"));
    // graph.add_decl(s_stack, SgLabel::Method, SgData::var("pop", "() -> int"));
    // graph.add_decl(s_stack, SgLabel::Method, SgData::var("count", "() -> int"));
    // graph.add_decl(s_stack, SgLabel::Method, SgData::var("sum", "() -> int"));

    graph.add_edge(s_stack, s_countable, SgLabel::Implement);
    graph.add_edge(s_stack, s_summable, SgLabel::Implement);
    // graph.add_decl(s_iterable, SgLabel::Method, SgData::var("iterator", "() -> Iterator"));

    graph.add_edge(s_countable, s_iterable, SgLabel::Extend);
    // graph.add_decl(s_countable, SgLabel::Method, SgData::var("count", "() -> int"));

    graph.add_edge(s_summable, s_iterable, SgLabel::Extend);
    // graph.add_decl(s_summable, SgLabel::Method, SgData::var("sum", "() -> int"));

    graph.as_uml_diagram("Interface Diamond Example", &render_options())
        .render_to_file("output/thesis_interface_diamond.puml")
        .unwrap();
}

#[test]
fn table_chain() {
    let head = GraphPattern::Linear(1);
    let pattern = GraphPattern::Linear(2);
    let tail = GraphPattern::Tree(2);
    let graph: Graph = GraphGenerator::default().with_patterns([head, pattern, tail]).build();
    graph.as_uml_diagram("Table Chain", &table_render_options())
        .render_to_file("output/thesis_table_chain.puml")
        .unwrap();
}

#[test]
fn table_circle() {
    let head = GraphPattern::Linear(1);
    let pattern = GraphPattern::Circle(2);
    let tail = GraphPattern::Tree(2);
    let graph: Graph = GraphGenerator::default().with_patterns([head, pattern, tail]).build();
    graph.as_uml_diagram("Table Circle", &table_render_options())
        .render_to_file("output/thesis_table_circle.puml")
        .unwrap();
}

#[test]
fn table_tree() {
    let head = GraphPattern::Linear(1);
    let pattern = GraphPattern::Tree(3);
    let tail = GraphPattern::Linear(1);
    let graph: Graph = GraphGenerator::default().with_patterns([head, pattern, tail]).build();
    graph.as_uml_diagram("Table Tree", &table_render_options())
        .render_to_file("output/thesis_table_tree.puml")
        .unwrap();
}

#[test]
fn table_diamond() {
    let head = GraphPattern::Linear(1);
    let pattern = GraphPattern::Diamond(3, 2);
    let tail = GraphPattern::Tree(2);
    let graph: Graph = GraphGenerator::default().with_patterns([head, pattern, tail]).build();
    graph.as_uml_diagram("Table Diamond", &table_render_options())
        .render_to_file("output/thesis_table_diamond.puml")
        .unwrap();
}