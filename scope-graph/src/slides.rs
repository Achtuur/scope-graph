#[cfg(test)]
mod slides_examples {
    use graphing::Renderer;

    use crate::{
        ColorSet, ForeGroundColor, SgData, SgLabel,
        generator::{GraphGenerator, GraphPattern},
        graph::{CachedScopeGraph, ScopeGraph},
        path::Path,
    };

    #[test]
    fn slides_example_query_2_data() {
        let pattern = [
            GraphPattern::Decl(SgData::var("x", "int")),
            GraphPattern::Decl(SgData::var("y", "int")),
            GraphPattern::Linear(1),
        ];

        let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();
        let mut diagram = graph.as_uml_diagram("graph", false);
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

        let mut diagram = graph.as_uml_diagram("cache", false);
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
        let diagram = graph.as_uml_diagram("graph", false);
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
        let diagram = graph.as_uml_diagram("graph", false);
        diagram
            .render_to_file("output/slides_example_query_2_data_longer/graph.puml")
            .unwrap();
    }

    #[test]
    fn slides_example_graph_no_data_diamond() {
        let pattern = [GraphPattern::Diamond(2)];

        let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

        graph
            .as_uml_diagram("Example Graph", false)
            .render_to_file("output/slides_example_no_data_diamond/graph.puml")
            .unwrap();
    }

    #[test]
    fn slides_example_graph_no_data_linear() {
        let pattern = [GraphPattern::Linear(2)];

        let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

        graph
            .as_uml_diagram("Example Graph", false)
            .render_to_file("output/slides_example_graph_no_data_linear/graph.puml")
            .unwrap();
    }

    #[test]
    fn slides_example_graph_no_data_tree() {
        let pattern = [GraphPattern::Tree(2)];

        let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

        graph
            .as_uml_diagram("Example Graph", false)
            .render_to_file("output/slides_example_graph_no_data_tree/graph.puml")
            .unwrap();
    }

    #[test]
    fn slides_example_graph_no_data_fanout() {
        let pattern = [
            GraphPattern::Decl(SgData::var("x", "int")),
            GraphPattern::Decl(SgData::var("y", "int")),
            GraphPattern::Decl(SgData::var("print()", "void -> void")),
        ];

        let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

        graph
            .as_uml_diagram("Example Graph", false)
            .render_to_file("output/slides_example_graph_no_data_fanout/graph.puml")
            .unwrap();
    }
}
