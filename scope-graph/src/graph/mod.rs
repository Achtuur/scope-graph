use std::{collections::HashMap, hash::Hash};

use graphing::{plantuml::{
    theme::{ElementCss, FontStyle, HorizontalAlignment, LineStyle, StyleSheet},
    EdgeDirection, NodeType, PlantUmlDiagram, PlantUmlItem,
}, Color};
use resolve::QueryResult;
use serde::{Deserialize, Serialize};

use crate::{
    data::ScopeGraphData, label::ScopeGraphLabel, order::LabelOrder, regex::dfs::RegexAutomata, scope::Scope, BackGroundEdgeColor, BackgroundColor, ColorSet, ForeGroundColor
};

mod base;
mod cached;
mod resolve;

pub use base::*;
pub use cached::*;

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
    pub children: Vec<Edge<Lbl>>,
    /// outgoing edges
    pub parents: Vec<Edge<Lbl>>,
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
            children: Vec::new(),
            parents: Vec::new(),
        }
    }

    pub fn children(&self) -> &[Edge<Lbl>] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<Edge<Lbl>> {
        &mut self.children
    }

    pub fn parents(&self) -> &[Edge<Lbl>] {
        &self.parents
    }

    pub fn parents_mut(&mut self) -> &mut Vec<Edge<Lbl>> {
        &mut self.parents
    }
}

pub type ScopeMap<Lbl, Data> = HashMap<Scope, ScopeData<Lbl, Data>>;

pub trait ScopeGraph<Lbl, Data>: std::fmt::Debug
where
    Lbl: ScopeGraphLabel,
    Data: ScopeGraphData,
{
    /// Add a scope to the graph with the given data.
    ///
    /// # Returns
    ///
    /// returns the scope that was added
    fn add_scope(&mut self, scope: Scope, data: Data) -> Scope;
    fn add_edge(&mut self, source: Scope, target: Scope, label: Lbl);

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
        path_regex: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_equiv: DEq,
        data_wellformedness: DWfd,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool,
        DWfd: for<'da> Fn(&'da Data) -> bool;

    /// 'r is lifetime of resolver
    fn query_proj<P, DProj, DEq>(
        &mut self,
        scope: Scope,
        path_regex: &RegexAutomata<Lbl>,
        order: &LabelOrder<Lbl>,
        data_proj: DProj,
        proj_wfd: P,
        data_equiv: DEq,
    ) -> Vec<QueryResult<Lbl, Data>>
    where
        P: Hash + Eq,
        DProj: for<'da> Fn(&'da Data) -> P,
        DEq: for<'da, 'db> Fn(&'da Data, &'db Data) -> bool;

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
    fn as_mmd(&self, title: &str) -> String {
        let mut mmd = format!(
            "---\n\
            title: \"{}\"\n\
            ---\n\
            flowchart LR\n\
            ",
            title
        );

        for (s, d) in self.scope_iter() {
            if d.data.variant_has_data() {
                mmd += &format!("\tscope_{}[\"{}\"]\n", s.0, d.data.render_string());
            } else {
                mmd += &format!("\tscope_{}((\"{}\"))\n", s.0, s.0);
            }
        }

        for (s, d) in self.scope_iter() {
            for edge in d.parents().iter() {
                mmd += &format!(
                    "scope_{} ==>|\"{}\"| scope_{}\n",
                    s.0,
                    edge.lbl().str(),
                    edge.to.0
                )
            }
        }
        mmd
    }

    fn as_uml_diagram(&self, display_cache: bool) -> PlantUmlDiagram {
        let mut style_sheet: StyleSheet = [
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
            ElementCss::new()
                .font_size(11)
                .as_class("cache-entry")
        ]
        .into();
        let fg = ForeGroundColor::stylesheet();
        let bg = BackgroundColor::stylesheet();
        let bg_line = BackGroundEdgeColor::stylesheet();
        style_sheet.merge(fg);
        style_sheet.merge(bg);
        style_sheet.merge(bg_line);

        let items = self.as_uml(display_cache);
        let mut diagram = PlantUmlDiagram::new("scope graph");
        diagram.extend(items);
        diagram.set_style_sheet(style_sheet);
        diagram
    }

    fn as_uml(&self, display_cache: bool) -> Vec<PlantUmlItem> {
        let mut items = self.generate_graph_uml();
        match display_cache {
            true => {
                items.extend(self.generate_cache_uml());
                items
            }
            false => items,
        }
    }

    fn generate_cache_uml<'a>(&'a self) -> Vec<PlantUmlItem>
    where
        Lbl: 'a,
        Data: 'a,
    {
        Vec::new()
    }

    fn generate_graph_uml(&self) -> Vec<PlantUmlItem> {
        let scope_nodes = self.scope_iter().map(|(s, d)| {
            let (node_type, class, contents) = match d.data.variant_has_data() {
                true => (
                    NodeType::Card,
                    "data-scope",
                    format!("{} > {}", s, d.data.render_string()),
                ),
                false => (NodeType::Node, "scope", s.to_string()),
            };
            PlantUmlItem::node(s.uml_id(), contents, node_type)
            .add_class(class)
            .add_class(BackgroundColor::get_class_name(s.0))
        });

        let edges = self.scope_iter().flat_map(move |(s, d)| {
            d.parents().iter().map(move |edge| {
                let dir = match self.scope_holds_data(edge.target()) {
                    true => EdgeDirection::Right,
                    false => EdgeDirection::Up,
                };

                PlantUmlItem::edge(s.uml_id(), edge.target().uml_id(), edge.lbl().str(), dir)
                    .add_class("scope_edge")
            })
        });

        scope_nodes.chain(edges).collect()
    }
}
