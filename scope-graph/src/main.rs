use std::{
    hint::black_box, io::{stdin, Write}, sync::Arc
};

use graphing::{plantuml::{theme::{ElementCss, FontFamily, FontStyle, HorizontalAlignment, LineStyle, PlantUmlStyleSheet}, EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem, PlantUmlItemKind}, Color, Renderer};
use scope_graph::{
    generator::{GraphGenerator, GraphPattern}, graph::{CachedScopeGraph, GraphRenderOptions, ScopeGraph}, order::LabelOrderBuilder, regex::{dfs::RegexAutomaton, Regex}, scope::Scope, BackGroundEdgeColor, BackgroundColor, ColorSet, ForeGroundColor, SgData, SgLabel, SgProjection, DRAW_CACHES, SAVE_GRAPH
};

pub type UsedScopeGraph = CachedScopeGraph<SgLabel, SgData>;

fn graph_builder() -> UsedScopeGraph {
    let graph = UsedScopeGraph::new();
    // let patterns = [
    //     GraphPattern::Linear(1),
    //     GraphPattern::Decl(SgData::var("x", "int")),
    //     // GraphPattern::Tree(100),
    //     // GraphPattern::ReverseTree(7),
    //     // GraphPattern::Decl(SgData::var("x1", "int")),
    //     // GraphPattern::Decl(SgData::var("x2", "int")),
    //     // GraphPattern::Decl(SgData::var("x3", "int")),
    //     // GraphPattern::Decl(SgData::var("x4", "int")),
    //     GraphPattern::Linear(3),
    //     GraphPattern::Linear(1),
    //     GraphPattern::Diamond(2, 1),
    //     GraphPattern::Decl(SgData::var("y", "int")),
    //     // GraphPattern::Decl(SgData::var("x", "int")),
    //     GraphPattern::Linear(10),
    // ];
    let patterns = [
        GraphPattern::Linear(1),
        GraphPattern::Decl(SgData::var("x", "int")),
        GraphPattern::Circle(3),
        // GraphPattern::Decl(SgData::var("x1", "int")),
        // GraphPattern::Decl(SgData::var("x2", "int")),
        // GraphPattern::Decl(SgData::var("x3", "int")),
        // GraphPattern::Decl(SgData::var("x4", "int")),
        // GraphPattern::Decl(SgData::var("x5", "int")),
        // GraphPattern::Decl(SgData::var("x6", "int")),
        // GraphPattern::Decl(SgData::var("x7", "int")),
        // GraphPattern::Decl(SgData::var("x8", "int")),
        // GraphPattern::Decl(SgData::var("x9", "int")),
        // GraphPattern::Decl(SgData::var("x10", "int")),
        // GraphPattern::Decl(SgData::var("x11", "int")),
        // GraphPattern::Decl(SgData::var("x12", "int")),
        GraphPattern::Linear(3),
        GraphPattern::Decl(SgData::var("y", "int")),
        // GraphPattern::Diamond(16, 1),
        GraphPattern::Tree(2),
        // GraphPattern::Join,
        GraphPattern::Linear(5),
    ];
    let graph = GraphGenerator::new(graph).with_patterns(patterns).build();
    graph
        .as_uml_diagram("graph", &GraphRenderOptions::default())
        .render_to_file("output/output0.puml")
        .unwrap();
    // graph
    //     .as_mmd_diagram("graph", DRAW_CACHES)
    //     .render_to_file("output/output0.md")
    //     .unwrap();
    graph
}

fn query_test(graph: &mut UsedScopeGraph) {
    let order = LabelOrderBuilder::new()
        .push(SgLabel::Declaration, SgLabel::Parent)
        .build();

    // P*D;
    let label_reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration);
    let matcher = RegexAutomaton::from_regex(label_reg.clone());
    matcher
        .to_uml()
        .render_to_file("output/regex.puml")
        .unwrap();
    matcher.to_mmd().render_to_file("output/regex.md").unwrap();

    let x_match: Arc<str> = Arc::from("x");
    let query_scope_set = [(x_match.clone(), vec![16]), (x_match.clone(), vec![11])];

    for (idx, set) in query_scope_set.into_iter().enumerate() {
        let title = format!(
            "Query sets {:?}, label_reg={}, label_order={}, proj={}",
            set,
            label_reg,
            order,
            SgProjection::VarName
        );

        let p = set.0;
        let start_scopes = set.1;
        let timer = std::time::Instant::now();
        let (res_uml, res_mmd) = start_scopes
            .into_iter()
            .flat_map(|s| {
                let scope = graph.first_scope_without_data(s).unwrap();
                graph.query_proj(scope, &matcher, &order, SgProjection::VarName, p.clone())
            })
            .fold((Vec::new(), Vec::new()), |(mut uml_acc, mut mmd_acc), r| {
                let fg_class = ForeGroundColor::next_class();
                let uml = r.path.as_uml(fg_class.clone(), true);
                let mmd = r.path.as_mmd(fg_class, true);
                uml_acc.extend(uml);
                mmd_acc.extend(mmd);
                (uml_acc, mmd_acc)
            });

        println!("{:?}", timer.elapsed());
        // mmd
        // let cache_mmd = graph.cache_path_mmd(11);
        // let mut mmd_diagram = graph.as_mmd_diagram(&title, DRAW_CACHES);
        // // mmd_diagram.extend(cache_mmd);
        // mmd_diagram.extend(res_mmd);

        // uml
        // let cache_uml = graph.cache_path_uml(11);
        let options = GraphRenderOptions {
            draw_caches: true,
            ..Default::default()
        };
        let mut uml_diagram = graph.as_uml_diagram(&title, &options);
        // uml_diagram.extend(cache_uml);
        uml_diagram.extend(res_uml);
        let fname = format!("output/output{}.puml", idx);
        uml_diagram.render_to_file(&fname).unwrap();
    }
}

fn circular_graph() -> UsedScopeGraph {
    let mut graph = CachedScopeGraph::new();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope_default();
    let s3 = graph.add_scope_default();
    graph.add_edge(s1, s2, SgLabel::Parent);
    graph.add_edge(s2, s1, SgLabel::Parent);
    graph.add_edge(s3, s1, SgLabel::Parent);
    let s4 = graph.add_decl(s1, SgLabel::Declaration, SgData::var("x", "int"));
    let s5 = graph.add_decl(s2, SgLabel::Declaration, SgData::var("y", "int"));
    graph
        .as_mmd_diagram("circular", DRAW_CACHES)
        .render_to_file("output/circular.md")
        .unwrap();
    graph
}

fn diamond_example() {
    let mut graph = UsedScopeGraph::new();
    let s0 = graph.add_scope_default();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope_default();
    let s3 = graph.add_scope_default();
    let s4 = graph.add_scope_default();
    graph.add_edge(s1, s0, SgLabel::Parent);
    graph.add_edge(s2, s0, SgLabel::Parent);
    graph.add_edge(s3, s1, SgLabel::Parent);
    graph.add_edge(s3, s2, SgLabel::Parent);
    graph.add_edge(s4, s3, SgLabel::Parent);
    let sd0 = graph.add_decl(s0, SgLabel::Declaration, SgData::var("x", "int"));

    
    graph
        .as_uml_diagram("circle sg", &GraphRenderOptions::default())
        .render_to_file("output/diamond0.puml")
        .unwrap();

    let reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration).compile();
    let label_order = LabelOrderBuilder::new()
        // .push(SgLabel::Declaration, SgLabel::Parent)
        .build();
    let wfd: Arc<str> = Arc::from("x");
    let timer = std::time::Instant::now();
    let env = graph.query_proj(
        s3,
        &reg,
        &label_order,
        SgProjection::VarName,
        wfd.clone(),
    );
    println!("diamond q1 {:?}", timer.elapsed());

    graph.as_uml_diagram("diamond example", &GraphRenderOptions {
        draw_caches: true,
        ..Default::default()
    }).render_to_file("output/diamond1.puml").unwrap();

    let timer = std::time::Instant::now();
    let env = graph.query_proj(
        s4,
        &reg,
        &label_order,
        SgProjection::VarName,
        wfd.clone(),
    );
    println!("diamond q2 {:?}", timer.elapsed());

    graph.as_uml_diagram("diamond example", &GraphRenderOptions {
        draw_caches: true,
        ..Default::default()
    }).render_to_file("output/diamond2.puml").unwrap();
}

fn aron_example() {
    let mut graph = UsedScopeGraph::new();
    let s0 = graph.add_scope_default();
    let s1 = graph.add_scope_default();
    let s2 = graph.add_scope_default();
    let s3 = graph.add_scope_default();
    graph.add_edge(s0, s1, SgLabel::Parent);
    graph.add_edge(s1, s2, SgLabel::Parent);
    graph.add_edge(s2, s3, SgLabel::Parent);
    graph.add_edge(s3, s0, SgLabel::Parent);
    let sd0 = graph.add_decl(s0, SgLabel::Declaration, SgData::var("x", "int"));
    let sd2 = graph.add_decl(s2, SgLabel::Declaration, SgData::var("x", "int"));

    graph
        .as_uml_diagram("circle sg", &GraphRenderOptions::default())
        .render_to_file("output/aron0.puml")
        .unwrap();

    let reg = Regex::concat(Regex::kleene(SgLabel::Parent), SgLabel::Declaration).compile();
    let label_order = LabelOrderBuilder::new()
        // .push(SgLabel::Declaration, SgLabel::Parent)
        .build();
    let wfd: Arc<str> = Arc::from("x");
    let env = graph.query_proj(
        s1,
        &reg,
        &label_order,
        SgProjection::VarName,
        wfd.clone(),
    );
    
    println!("env: {0:?}", env);

    let mut style_sheet: PlantUmlStyleSheet = [
            ElementCss::new()
                .background_color(Color::new_rgb(242, 232, 230))
                .font_family(FontFamily::Monospace)
                .as_selector("element"),
            ElementCss::new()
                .line_color(Color::BLACK)
                .as_selector("arrow"),
            ElementCss::new()
                .font_size(24)
                .font_style(FontStyle::Bold)
                .round_corner(1000)
                .horizontal_alignment(HorizontalAlignment::Center)
                .as_class("scope"),
            ElementCss::new()
                .font_size(24)
                .font_style(FontStyle::Bold)
                .round_corner(10)
                .shadowing(1)
                .background_color(Color::new_rgb(245, 229, 220))
                .as_class("data-scope"),
            ElementCss::new()
                .line_thickness(1.25)
                .font_size(16)
                .as_class("scope-edge"),
            ElementCss::new()
                .line_style(LineStyle::Dashed)
                .as_class("query-edge"),
            ElementCss::new()
                .line_style(LineStyle::Dotted)
                .line_color(Color::LIGHT_GRAY)
                .as_class("cache-edge"),
            ElementCss::new().font_size(11).as_class("cache-entry"),
        ]
        .into();
        let fg = ForeGroundColor::uml_stylesheet();
        let bg = BackgroundColor::uml_stylesheet();
        let bg_line = BackGroundEdgeColor::uml_stylesheet();
        style_sheet.merge(fg);
        style_sheet.merge(bg);
        style_sheet.merge(bg_line);

    let mut diagram = PlantUmlDiagram::new("1st query");
    diagram.set_style_sheet(style_sheet);
    diagram.push(PlantUmlItem::node(s0.uml_id(), "0", NodeType::Node));
    diagram.push(PlantUmlItem::node(s1.uml_id(), "1", NodeType::Node));
    diagram.push(PlantUmlItem::node(s2.uml_id(), "2", NodeType::Node));
    diagram.push(PlantUmlItem::node(s3.uml_id(), "3", NodeType::Node));
    diagram.push(PlantUmlItem::node(sd0.uml_id(), "4 x: int", NodeType::Card));
    diagram.push(PlantUmlItem::node(sd2.uml_id(), "5 x: int", NodeType::Card));

    diagram.push(PlantUmlItem::edge(s0.uml_id(), s1.uml_id(), "P", EdgeDirection::Right));
    diagram.push(PlantUmlItem::edge(s1.uml_id(), s2.uml_id(), "P", EdgeDirection::Bottom));
    diagram.push(PlantUmlItem::edge(s2.uml_id(), s3.uml_id(), "P", EdgeDirection::Left));
    diagram.push(PlantUmlItem::edge(s3.uml_id(), s0.uml_id(), "P", EdgeDirection::Up));

    diagram.push(PlantUmlItem::edge(s0.uml_id(), sd0.uml_id(), "D", EdgeDirection::Left));
    diagram.push(PlantUmlItem::edge(s2.uml_id(), sd2.uml_id(), "D", EdgeDirection::Right));

    let mut cache = graph.generate_cache_uml();
    for item in &mut cache {
        match item.node_id() {
            id if id == s0.uml_id() || id == s1.uml_id() => item.set_direction(EdgeDirection::Up),
            id if id == s2.uml_id() || id == s3.uml_id() => item.set_direction(EdgeDirection::Bottom),
            _ => ()
        }
    }

    let mut d1 = diagram.clone();
    d1.extend(cache);

    let q_uml = env
        .into_iter()
        .flat_map(|r| r.path.as_uml(ForeGroundColor::next_class(), true))
        .collect::<Vec<_>>();
    d1.extend(q_uml);

    d1.push(PlantUmlItem::note(s1.uml_id(), format!("Query 1 start in scope {s1}, looking for {wfd}"), EdgeDirection::Right));
    d1.render_to_file("output/aron1.puml").unwrap();

    let wfd: Arc<str> = Arc::from("x");
    let env = graph.query_proj(
        s3,
        &reg,
        &label_order,
        SgProjection::VarName,
        wfd.clone(),
    );

    let mut d2 = diagram.clone();
    let mut cache = graph.generate_cache_uml();
    for item in &mut cache {
        match item.node_id() {
            id if id == s0.uml_id() || id == s1.uml_id() => item.set_direction(EdgeDirection::Up),
            id if id == s2.uml_id() || id == s3.uml_id() => item.set_direction(EdgeDirection::Bottom),
            _ => ()
        }
    }
    d2.extend(cache);
    let q_uml = env
        .into_iter()
        .flat_map(|r| r.path.as_uml(ForeGroundColor::next_class(), true))
        .collect::<Vec<_>>();
    d2.extend(q_uml);

    d2.push(PlantUmlItem::note(s3.uml_id(), format!("Query 2 start in scope {s3}, looking for {wfd}"), EdgeDirection::Left));
    d2.render_to_file("output/aron2.puml").unwrap();
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();
    aron_example();
    Scope::reset_counter();

    diamond_example();
    Scope::reset_counter();

    let mut graph = graph_builder();
    query_test(&mut graph);
}

fn save_graph(graph: &UsedScopeGraph, fname: &str) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fname)
        .unwrap();
    tracing::info!("Writing to file {}", fname);
    serde_json::to_writer(file, graph).unwrap();
}


#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use deepsize::DeepSizeOf;

    use super::*;

    #[test]
    fn test_deepsize() {
        let mut map = HashMap::new();
        println!("{}", map.deep_size_of());
        map.insert("key".to_string(), "value".to_string());
        println!("{}", map.deep_size_of());

        let mut v = Vec::new();
        println!("v.deep_size_of(): {0:?}", v.deep_size_of());
        for i in 0..8_u8 {
            v.push(i);
        }
        println!("v.deep_size_of(): {0:?}", v.deep_size_of());

    }
}