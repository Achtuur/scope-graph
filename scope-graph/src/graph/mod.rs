use std::collections::HashMap;

use graphing::{
    Color,
    mermaid::{
        MermaidChartDirection, MermaidDiagram, MermaidStyleSheet,
        item::{ItemShape, MermaidItem},
        theme::{AnimationSpeed, AnimationStyle, EdgeType, ElementStyle, Size},
    },
    plantuml::{
        EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem,
        theme::{ElementCss, FontStyle, HorizontalAlignment, LineStyle, PlantUmlStyleSheet},
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    BackGroundEdgeColor, BackgroundColor, ColorSet, ForeGroundColor, data::ScopeGraphData,
    label::ScopeGraphLabel, order::LabelOrder, projection::ScopeGraphDataProjection,
    regex::dfs::RegexAutomaton, scope::Scope,
};

// mod base;
mod cached;
mod resolve;

// pub use base::*;
pub use cached::*;
pub use resolve::QueryResult;

/// Bi-directional edge between two scopes
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
        tracing::debug!(
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

    fn as_uml_diagram(&self, title: &str, draw_caches: bool) -> PlantUmlDiagram {
        let mut style_sheet: PlantUmlStyleSheet = [
            ElementCss::new()
                .background_color(Color::new_rgb(242, 232, 230))
                .as_selector("element"),
            ElementCss::new()
                .line_color(Color::BLACK)
                .as_selector("arrow"),
            ElementCss::new()
                .font_size(16)
                .font_style(FontStyle::Bold)
                .round_corner(0)
                .horizontal_alignment(HorizontalAlignment::Center)
                .as_class("scope"),
            ElementCss::new()
                .font_size(18)
                .font_style(FontStyle::Bold)
                .round_corner(10)
                .shadowing(1)
                .background_color(Color::new_rgb(242, 232, 175))
                .as_class("data-scope"),
            ElementCss::new()
                .line_thickness(1.25)
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

        let mut diagram = PlantUmlDiagram::new("scope graph");
        diagram.set_style_sheet(style_sheet);
        diagram.extend(self.generate_graph_uml());
        if draw_caches {
            diagram.extend(self.generate_cache_uml());
        }
        diagram
    }

    fn generate_graph_uml(&self) -> Vec<PlantUmlItem> {
        let scope_nodes = self.scope_iter().map(|(s, d)| {
            let (node_type, class, contents) = match d.data.variant_has_data() {
                true => (
                    NodeType::Card,
                    "data-scope",
                    format!("{} ⊢ {}", s, d.data.render_string()),
                ),
                false => (NodeType::Node, "scope", s.to_string()),
            };
            PlantUmlItem::node(s.uml_id(), contents, node_type)
                .add_class(class)
                .add_class(BackgroundColor::get_class_name(s.0))
        });

        let mut decl_dir = false;

        let edges = self.scope_iter().flat_map(move |(s, d)| {
            d.outgoing().iter().map(move |edge| {
                let dir = match self.scope_holds_data(edge.target()) {
                    true =>  {
                        decl_dir = !decl_dir;
                        match decl_dir {
                            true => EdgeDirection::Left,
                            false => EdgeDirection::Right,
                        }
                    },
                    false => EdgeDirection::Up,
                };

                PlantUmlItem::edge(s.uml_id(), edge.target().uml_id(), edge.lbl().char(), dir)
                    .add_class("scope_edge")
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
