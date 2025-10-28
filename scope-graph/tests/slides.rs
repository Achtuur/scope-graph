use graphing::Renderer;
use scope_graph::graph::{GraphRenderOptions, LabelRenderStyle};

use scope_graph::{
    generator::{GraphGenerator, GraphPattern}, graph::{CachedScopeGraph, ScopeGraph}, path::Path, scope::Scope, ColorSet, ForeGroundColor, SgData, SgLabel
};

pub fn render_options() -> GraphRenderOptions {
    GraphRenderOptions {
        draw_caches: false,
        draw_labels: LabelRenderStyle::None,
        draw_types: false,
        draw_node_label: true,
        draw_colors: false,
    }
}

#[test]
fn slides_example_query_2_data() {
    let pattern = [
        GraphPattern::Decl(SgData::var("x", "int")),
        GraphPattern::Decl(SgData::var("y", "int")),
        GraphPattern::Linear(1),
    ];

    let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();
    let mut diagram = graph.as_uml_diagram("graph", &render_options());
    diagram
        .render_to_file("output/slides_example_query_2_data/graph.puml")
        .unwrap();

    let path = Path::start(3)
        .step(SgLabel::Parent, 0, 0)
        .step(SgLabel::Declaration, 1, 0);
    let class = ForeGroundColor::next_class();
    diagram.extend(path.as_uml(class, false));
    diagram.set_title("path1");
    diagram
        .render_to_file("output/slides_example_query_2_data/path1.puml")
        .unwrap();

    let path2 = Path::start(3)
        .step(SgLabel::Parent, 0, 0)
        .step(SgLabel::Declaration, 2, 0);
    let class = ForeGroundColor::next_class();
    diagram.extend(path2.as_uml(class, false));
    diagram.set_title("path2");
    diagram
        .render_to_file("output/slides_example_query_2_data/path2.puml")
        .unwrap();

    let mut diagram = graph.as_uml_diagram("cache", &render_options());
    diagram.extend(path.as_uml(ForeGroundColor::next_class(), false));
    diagram.extend(path2.as_uml(ForeGroundColor::next_class(), true));
    diagram
        .render_to_file("output/slides_example_query_2_data/cache.puml")
        .unwrap();

    // // cache
    // let reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration).compile();
    // let order = LabelOrderBuilder::new().push(SgLabel::Declaration, SgLabel::Parent).build();

    // let envs = graph.query_proj(Scope(3), &reg, &order, SgProjection::VarName, "x".into());
    // let x_env = envs.first().unwrap();

    // let path = x_env.path.as_uml(ForeGroundColor::next_class(), true);
    // let mut diagram = graph.as_uml_diagram("graph+cache", true);
    // diagram.extend(path);
    // diagram.set_title("x path");
    // diagram.render_to_file("output/slides_example_query_2_data/cache.puml").unwrap();
}

#[test]
fn slides_example_query_2_data_long() {
    let pattern = [
        GraphPattern::Decl(SgData::var("x", "int")),
        GraphPattern::Decl(SgData::var("y", "int")),
        GraphPattern::Linear(5),
    ];

    let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();
    let diagram = graph.as_uml_diagram("graph", &render_options());
    diagram
        .render_to_file("output/slides_example_query_2_data_long/graph.puml")
        .unwrap();
}

#[test]
fn slides_example_query_2_data_even_longer() {
    let pattern = [
        GraphPattern::Decl(SgData::var("x", "int")),
        GraphPattern::Decl(SgData::var("y", "int")),
        GraphPattern::Linear(10),
    ];

    let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();
    let diagram = graph.as_uml_diagram("graph", &render_options());
    diagram
        .render_to_file("output/slides_example_query_2_data_longer/graph.puml")
        .unwrap();
}

#[test]
fn slides_example_graph_no_data_diamond() {
    let pattern = [GraphPattern::Diamond(2, 4)];

    let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

    graph
        .as_uml_diagram("Example Graph", &render_options())
        .render_to_file("output/slides_example_no_data_diamond/graph.puml")
        .unwrap();
}

#[test]
fn slides_example_graph_no_data_linear() {
    let pattern = [GraphPattern::Linear(2)];

    let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

    graph
        .as_uml_diagram("Example Graph", &render_options())
        .render_to_file("output/slides_example_graph_no_data_linear/graph.puml")
        .unwrap();
}

#[test]
fn slides_example_graph_no_data_tree() {
    let pattern = [GraphPattern::Tree(2)];

    let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

    graph
        .as_uml_diagram("Example Graph", &render_options())
        .render_to_file("output/slides_example_graph_no_data_tree/graph.puml")
        .unwrap();
}

#[test]
fn slides_example_graph_no_data_fanout() {
    let pattern = [
        GraphPattern::Decl(SgData::var("x", "")),
        GraphPattern::Decl(SgData::var("y", "")),
        GraphPattern::Decl(SgData::var("print()", "")),
    ];

    let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

    graph
        .as_uml_diagram("Example Graph", &render_options())
        .render_to_file("output/slides_example_graph_no_data_fanout/graph.puml")
        .unwrap();
}

#[test]
fn slides_example_graph_data_varying() {
    for n in [2, 4, 8] {
        for m in [1, 2] {
            let pattern = [GraphPattern::Diamond(n, m)];
            let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();
            graph
            .as_uml_diagram(&format!("Diamond {n}x{m}"), &render_options())
            .render_to_file(&format!("output/slides_example_graph_data_varying/graph{n}x{m}.puml"))
            .unwrap();
            Scope::reset_counter();
        }
    }
}

/// int i, let i, class i; example
#[test]
fn slide_page1_example() {
    let mut graph = CachedScopeGraph::<SgLabel, SgData>::new();
    let s0 = graph.add_scope_default();
    graph.add_decl(s0, SgLabel::Declaration, SgData::var("i", "..."));
    graph
            .as_uml_diagram("page1 example", &render_options())
            .render_to_file("output/page1_example.puml")
            .unwrap();
}

#[test]
fn slide_page1_example_struct() {
    let mut graph = CachedScopeGraph::<SgLabel, SgData>::new();
    let s_struct = graph.add_scope(Scope::new(), SgData::var("i", "struct|class"));
    graph.add_decl(s_struct, SgLabel::Declaration, SgData::var("x", "int|i32"));
    graph
            .as_uml_diagram("page1 example", &render_options())
            .render_to_file("output/page1_example_struct.puml")
            .unwrap();
}

#[test]
fn slide_page1_example_ext() {
    let mut graph = CachedScopeGraph::<SgLabel, SgData>::new();
    let s_struct = graph.add_scope(Scope::new(), SgData::var("i", "class"));
    let s_struct2 = graph.add_scope(Scope::new(), SgData::var("j", "class"));
    graph.add_edge(s_struct, s_struct2, SgLabel::Extend);
    graph
            .as_uml_diagram("page1 example", &render_options())
            .render_to_file("output/page1_example_ext.puml")
            .unwrap();
}

#[test]
fn slide_example_benchmark() {
    let mut graph = CachedScopeGraph::<SgLabel, SgData>::new();
    // head
    let s0 = graph.add_scope_default();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope_default();
    graph.add_edge(s1, s0, SgLabel::Parent);
    graph.add_edge(s2, s1, SgLabel::Parent);
    graph.add_decl(s0, SgLabel::Declaration, SgData::var("x_0", "int"));
    graph.add_decl(s2, SgLabel::Declaration, SgData::var("x_1", "int"));

    // pattern
    // Scope::reset_counter();
    let s_diamond_head = graph.add_scope_default();
    let s_diamond_left = graph.add_scope_default();
    let s_diamond_right = graph.add_scope_default();
    let s_diamond_bottom = graph.add_scope_default();
    graph.add_edge(s_diamond_head, s2, SgLabel::Parent);
    graph.add_edge(s_diamond_left, s_diamond_head, SgLabel::Parent);
    graph.add_edge(s_diamond_right, s_diamond_head, SgLabel::Parent);
    graph.add_edge(s_diamond_bottom, s_diamond_left, SgLabel::Parent);
    graph.add_edge(s_diamond_bottom, s_diamond_right, SgLabel::Parent);

    // tail
    // Scope::reset_counter();
    let s3 = graph.add_scope_default();
    let s4 = graph.add_scope_default();
    graph.add_edge(s3, s_diamond_bottom, SgLabel::Parent);
    graph.add_edge(s4, s3, SgLabel::Parent);

    graph
            .as_uml_diagram("example benchmark", &render_options())
            .render_to_file("output/example_benchmark.puml")
            .unwrap();
}
