#[cfg(test)]
mod slides_examples {
    use graphing::Renderer;

    use crate::{generator::{GraphGenerator, GraphPattern}, graph::{CachedScopeGraph, ScopeGraph}, path::Path, scope::Scope, ColorSet, ForeGroundColor, SgData, SgLabel};

    #[test]
    fn slides_example_query_2_data() {
        let pattern = [
            GraphPattern::Decl(SgData::var("x", "int")),
            GraphPattern::Decl(SgData::var("y", "int")),
            GraphPattern::Linear(1),
        ];

        let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

        let path = Path::start(3)
            .step(SgLabel::Parent, 0, 0)
            .step(SgLabel::Declaration, 1, 0);

        let mut diagram = graph.as_uml_diagram("Example Graph", false);

        diagram.render_to_file("output/slides_example_query_2_data.puml").unwrap();

        let class = ForeGroundColor::next_class();
        diagram.extend(path.as_uml(class, false));
        diagram.render_to_file("output/slides_example_query_2_data_path.puml").unwrap();
    }

    #[test]
    fn slides_example_query_2_data_diamond() {
        let pattern = [
            GraphPattern::Decl(SgData::var("x", "int")),
            GraphPattern::Decl(SgData::var("y", "int")),
            GraphPattern::Diamond(2),
        ];

        let graph: CachedScopeGraph<_, _> = GraphGenerator::from_pattern_iter(pattern).build();

        graph.as_uml_diagram("Example Graph", false)
            .render_to_file("output/slides_example_query_2_data_diamond.puml")
            .unwrap();
    }
}
