


#[cfg(test)]
mod slides_examples {
    use graphing::Renderer;
    use crate::graph::{GraphRenderOptions, LabelRenderStyle};

    use crate::{
        generator::{GraphGenerator, GraphPattern}, graph::{CachedScopeGraph, ScopeGraph}, path::Path, scope::Scope, ColorSet, ForeGroundColor, SgData, SgLabel
    };

    pub fn render_options() -> GraphRenderOptions {
        GraphRenderOptions {
            draw_caches: false,
            draw_labels: LabelRenderStyle::None,
            draw_types: false,
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
}

mod thesis_images {
    use graphing::Renderer;

    use crate::{graph::{CachedScopeGraph, GraphRenderOptions, LabelRenderStyle, ScopeGraph}, scope::Scope, SgData, SgLabel};

    pub fn render_options() -> GraphRenderOptions {
        GraphRenderOptions {
            draw_caches: false,
            draw_labels: LabelRenderStyle::Long,
            draw_types: true,
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
}
