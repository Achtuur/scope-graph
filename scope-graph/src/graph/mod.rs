use std::{collections::HashMap, sync::{Mutex, OnceLock}};

use deepsize::DeepSizeOf;
use graphing::{
    mermaid::{
        item::{ItemShape, MermaidItem}, theme::{AnimationSpeed, AnimationStyle, EdgeType, ElementStyle, Size}, MermaidChartDirection, MermaidDiagram, MermaidStyleSheet
    }, plantuml::{
        theme::{ElementCss, FontFamily, FontStyle, HorizontalAlignment, LineStyle, PlantUmlStyleSheet}, EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem
    }, Color
};
use serde::{Deserialize, Serialize};

use crate::{
    data::ScopeGraphData, debug_tracing, graph::circle::{CircleMatch, CircleMatcher}, label::ScopeGraphLabel, order::LabelOrder, projection::ScopeGraphDataProjection, regex::dfs::RegexAutomaton, scope::Scope, BackGroundEdgeColor, BackgroundColor, ColorSet, ForeGroundColor, DRAW_CACHES
};

// mod base;
mod cached;
mod resolve;
mod circle;

// pub use base::*;
pub use cached::*;
pub use resolve::{QueryResult, QueryStats};


#[derive(Clone, Copy, Default, Debug)]
pub enum LabelRenderStyle {
    None,
    #[default]
    Short,
    Long,
}

#[derive(Debug)]
pub struct GraphRenderOptions {
    pub draw_caches: bool,
    pub draw_labels: LabelRenderStyle,
    pub draw_types: bool,
    pub draw_node_label: bool,
    pub draw_colors: bool,
}

impl std::default::Default for GraphRenderOptions {
    fn default() -> Self {
        Self {
            draw_caches: DRAW_CACHES,
            draw_labels: LabelRenderStyle::default(),
            draw_types: true,
            draw_node_label: true,
            draw_colors: true,
        }
    }
}

/// Bi-directional edge between two scopes
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[derive(DeepSizeOf)]
pub struct Edge<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub to: (Scope, Lbl),
}

impl<Lbl: ScopeGraphLabel> Edge<Lbl> {
    pub fn new(scope: Scope, label: Lbl) -> Self {
        Self { to: (scope, label) }
    }

    pub fn target(&self) -> Scope {
        self.to.0
    }

    pub fn lbl(&self) -> &Lbl {
        &self.to.1
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(DeepSizeOf)]
pub struct ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData,
{
    // pub edges: Vec<Edge<Lbl>>,
    /// incoming edges
    pub incoming: Vec<Edge<Lbl>>,
    /// outgoing edges
    pub outgoing: Vec<Edge<Lbl>>,
    pub data: Data,
}

impl<Lbl, Data> ScopeData<Lbl, Data>
where
    Lbl: ScopeGraphLabel + Clone,
    Data: ScopeGraphData,
{
    pub fn new(data: Data) -> Self {
        Self {
            data,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }

    pub fn incoming(&self) -> &[Edge<Lbl>] {
        &self.incoming
    }

    pub fn incoming_mut(&mut self) -> &mut Vec<Edge<Lbl>> {
        &mut self.incoming
    }

    pub fn outgoing(&self) -> &[Edge<Lbl>] {
        &self.outgoing
    }

    pub fn outgoing_mut(&mut self) -> &mut Vec<Edge<Lbl>> {
        &mut self.outgoing
    }
}

pub type ScopeMap<Lbl, Data> = HashMap<Scope, ScopeData<Lbl, Data>>;


pub(crate) fn scope_is_part_of_cycle<Lbl, Data>(map: &ScopeMap<Lbl, Data>, scope: Scope) -> bool
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    CircleMatcher::scope_is_in_cycle(map, scope)
}

pub trait ScopeGraph<Lbl, Data>
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    fn reset_cache(&mut self);

    /// Add a scope to the graph with the given data.
    ///
    /// # Returns
    ///
    /// returns the scope that was added
    fn add_scope(&mut self, scope: Scope, data: Data) -> Scope;
    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl);

    fn add_scope_default(&mut self) -> Scope {
        self.add_scope(Scope::new(), Data::default())
    }

    fn add_decl(&mut self, source: Scope, label: Lbl, data: Data) -> Scope {
        debug_tracing!(debug,
            "Adding decl: {} with label: {} and data: {}",
            source,
            label,
            data
        );
        let decl_scope = Scope::new();
        self.add_scope(decl_scope, data);
        self.add_edge(source, decl_scope, label);
        decl_scope
    }

    /// 'r is lifetime of resolver
    fn query<DEq, DWfd>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomaton<Lbl>,
        order: &LabelOrder<Lbl>,
        data_equiv: DEq,
        data_wellformedness: DWfd,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
        DWfd: for<'da> Fn(&'da Data) -> bool;

    /// Query using a projection function and a wellformedness value for the projected data
    fn query_proj<Proj>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomaton<Lbl>,
        order: &LabelOrder<Lbl>,
        data_proj: Proj,
        proj_wfd: Proj::Output,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        Proj: ScopeGraphDataProjection<Data>;

    fn get_scope(&self, scope: Scope) -> Option<&ScopeData<Lbl, Data>>;

    // stuff for generating graphs below
    fn scope_iter<'a>(&'a self) -> impl Iterator<Item = (&'a Scope, &'a ScopeData<Lbl, Data>)>
    where
        Lbl: 'a,
        Data: 'a;

    /// Extend self with scopes and edges from other
    fn extend(&mut self, other: Self);

    /// Finds a scope, is here for debugging
    fn find_scope(&self, scope_num: usize) -> Option<Scope> {
        self.scope_iter()
            .find_map(|(s, _)| (s.0 == scope_num).then_some(*s))
    }
    /// Finds a scope without data, is here for debugging
    fn first_scope_without_data(&self, scope_num: usize) -> Option<Scope> {
        let mut non_data_scopes = self
            .scope_iter()
            .filter(|(s, d)| s.0 >= scope_num && !d.data.variant_has_data())
            .map(|(s, _)| *s)
            .collect::<Vec<_>>();
        non_data_scopes.sort_by_key(|s| s.0);
        non_data_scopes.first().copied()
    }

    fn scope_holds_data(&self, scope: Scope) -> bool;

    fn scope_is_part_of_cycle(&self, scope: Scope) -> bool {
        // todo: implement
        false
    }

    fn as_uml_diagram(&self, title: &str, options: &GraphRenderOptions) -> PlantUmlDiagram {
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

        let mut diagram = PlantUmlDiagram::new(title);
        diagram.set_style_sheet(style_sheet);
        diagram.extend(self.generate_graph_uml(options));
        if options.draw_caches {
            diagram.extend(self.generate_cache_uml());
        }
        diagram
    }

    fn generate_graph_uml(&self, options: &GraphRenderOptions) -> Vec<PlantUmlItem> {
        let scope_nodes = self.scope_iter().map(|(s, d)| {
            let (node_type, class, contents) = match d.data.variant_has_data() {
                true => {
                    let d_str = match options.draw_types {
                        true => d.data.render_with_type(),
                        false => d.data.render_string(),
                    };
                    (NodeType::Card, "data-scope", format!("{} ⊢ {}", s, d_str))
                },
                false => {
                    let contents = if options.draw_node_label {
                        s.to_string()
                    } else {
                        String::from("0") // empty is not possible ugh
                    };
                    (NodeType::Card, "scope", contents)
                },
            };
            let mut node = PlantUmlItem::node(s.uml_id(), contents, node_type).add_class(class);
            if options.draw_colors {
                node = node.add_class(BackgroundColor::get_class_name(s.0));
            }
            node
        });

        let mut decl_dir = 0;

        let edges = self.scope_iter().flat_map(move |(s, d)| {
            d.outgoing().iter().map(move |edge| {
                let dir = match self.scope_holds_data(edge.target()) {
                    true => {
                        decl_dir = (decl_dir + 1) % 4;
                        match decl_dir {
                            0 => EdgeDirection::Bottom,
                            1 => EdgeDirection::Left,
                            2 => EdgeDirection::Right,
                            _ => EdgeDirection::Up,
                        }
                    }
                    false => EdgeDirection::Up,
                };

                let lbl = match options.draw_labels {
                    LabelRenderStyle::None => String::new(),
                    LabelRenderStyle::Short => edge.lbl().char().to_string(),
                    LabelRenderStyle::Long => edge.lbl().str().to_string(),
                };

                PlantUmlItem::edge(s.uml_id(), edge.target().uml_id(), lbl, dir)
                    .add_class("scope-edge")
            })
        });

        scope_nodes.chain(edges).collect()
    }

    fn generate_cache_uml(&self) -> Vec<PlantUmlItem> {
        Vec::new()
    }

    fn as_mmd_diagram(&self, title: &str, draw_caches: bool) -> MermaidDiagram {
        let mut style_sheet = MermaidStyleSheet::new()
            .with_class(
                "scope",
                ElementStyle::new()
                    .line_color(Color::DARK_GRAY)
                    .font_size(Size::Pt(18))
                    .margin(Size::Px(5))
                    .padding(Size::Px(5)),
            )
            .with_class(
                "data-scope",
                ElementStyle::new()
                    .line_color(Color::BLACK)
                    .background_color(Color::new_rgb(242, 232, 175)),
            )
            .with_class("scope-edge", ElementStyle::new().line_thickness(2.5))
            .with_class(
                "query-edge",
                ElementStyle::new()
                    .line_thickness(1.5)
                    .animation_style(AnimationStyle::Linear)
                    .animation_speed(AnimationSpeed::Slow),
            )
            .with_class("cache-entry", ElementStyle::new().font_size(Size::Pt(8)))
            .with_class("cache-edge", ElementStyle::new());

        let fg = ForeGroundColor::mmd_stylesheet();
        let bg = BackgroundColor::mmd_stylesheet();
        let bg_line = BackGroundEdgeColor::mmd_stylesheet();
        style_sheet.merge(fg);
        style_sheet.merge(bg);
        style_sheet.merge(bg_line);

        let mut diagram = MermaidDiagram::new(title);
        diagram.set_style_sheet(style_sheet);
        diagram.set_direction(MermaidChartDirection::BottomTop);
        diagram.extend(self.generate_graph_mmd());
        if draw_caches {
            diagram.extend(self.generate_cache_mmd());
        }
        diagram
    }

    fn generate_cache_mmd(&self) -> Vec<MermaidItem> {
        Vec::new()
    }

    fn generate_graph_mmd(&self) -> Vec<MermaidItem> {
        let scope_nodes = self
            .scope_iter()
            .map(|(s, d)| match d.data.variant_has_data() {
                true => {
                    let contents = format!("{} ⊢ {}", s, d.data.render_string());
                    MermaidItem::node(s.uml_id(), contents, ItemShape::Rounded)
                        .add_class("data-scope")
                }
                false => {
                    let contents = s.to_string();
                    MermaidItem::node(s.uml_id(), contents, ItemShape::Circle)
                        .add_class("scope")
                        .add_class(BackgroundColor::get_class_name(s.0))
                }
            });

        let edges = self.scope_iter().flat_map(move |(s, d)| {
            d.outgoing().iter().map(move |edge| {
                MermaidItem::edge(
                    s.uml_id(),
                    edge.target().uml_id(),
                    edge.lbl().char(),
                    EdgeType::Thick,
                )
                .add_class("scope-edge")
            })
        });

        scope_nodes.chain(edges).collect()
    }
}
