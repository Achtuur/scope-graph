use std::sync::{Arc, atomic::AtomicUsize};

use data::ScopeGraphData;
use deepsize::DeepSizeOf;
use graphing::{
    Color,
    mermaid::{MermaidStyleSheet, theme::ElementStyle},
    plantuml::theme::{ElementCss, PlantUmlStyleSheet},
};
use label::ScopeGraphLabel;
use projection::ScopeGraphDataProjection;
use scopegraphs::{
    completeness::UncheckedCompleteness,
    render::{RenderScopeData, RenderScopeLabel},
};
use serde::{Deserialize, Serialize};

pub use graphing;

pub mod bench_util;

pub mod label;
pub mod path;
pub mod scope;

pub mod data;
pub mod generator;
pub mod graph;
pub mod order;
pub mod projection;
pub mod regex;
mod slides;
pub mod util;

/// Enable circular path check in cached resolver
pub const DO_CIRCLE_CHECK: bool = true;

pub const DO_SINGLE_EDGE_CHECK: bool = false;

/// Draw caches in the graph
pub const DRAW_CACHES: bool = true;
/// Draw memory addresses for the paths
pub const DRAW_MEM_ADDR: bool = false;
/// Prompt to save the graph
pub const SAVE_GRAPH: bool = false;



pub struct ForeGroundColor;
pub struct BackgroundColor;
pub struct BackGroundEdgeColor;

const FG_COLORS: &[Color] = &[
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::YELLOW,
    Color::PURPLE,
    Color::ORANGE,
    Color::CYAN,
];

const BG_COLORS: &[Color] = &[
    Color::LIGHT_RED,
    Color::LIGHT_GREEN,
    Color::LIGHT_BLUE,
    Color::LIGHT_YELLOW,
    Color::LIGHT_PURPLE,
    Color::LIGHT_ORANGE,
    Color::LIGHT_CYAN,
];

pub static COLOR_POINTER: AtomicUsize = AtomicUsize::new(0);

pub trait ColorSet {
    const COLORS: &'static [Color];

    fn get_class_name(idx: usize) -> String;

    fn get_uml_css(idx: usize) -> ElementCss;
    fn get_mmd_css(idx: usize) -> ElementStyle;

    fn next_class() -> String {
        let idx = COLOR_POINTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self::get_class_name(idx)
    }

    fn next_color() -> Color {
        let idx = COLOR_POINTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self::COLORS[idx % Self::COLORS.len()]
    }

    fn get_color(idx: usize) -> Color {
        Self::COLORS[idx % Self::COLORS.len()]
    }

    fn class_iter() -> impl Iterator<Item = (String, Color)> {
        (0..Self::COLORS.len()).map(|idx| {
            let name = Self::get_class_name(idx);
            let color = Self::get_color(idx);
            (name, color)
        })
    }

    fn class_name_iter() -> impl Iterator<Item = String> {
        (0..Self::COLORS.len()).map(Self::get_class_name)
    }

    fn uml_stylesheet() -> PlantUmlStyleSheet {
        (0..Self::COLORS.len())
            .map(|i| {
                let class_name = Self::get_class_name(i);
                Self::get_uml_css(i).as_class(class_name)
            })
            .collect()
    }

    fn mmd_stylesheet() -> MermaidStyleSheet {
        (0..Self::COLORS.len())
            .map(|i| {
                let class_name = Self::get_class_name(i);
                (class_name, Self::get_mmd_css(i))
            })
            .collect()
    }
}

impl ColorSet for ForeGroundColor {
    const COLORS: &[Color] = FG_COLORS;

    fn get_class_name(idx: usize) -> String {
        format!("foreground-{}", idx % Self::COLORS.len())
    }

    fn get_uml_css(idx: usize) -> ElementCss {
        let color = Self::get_color(idx);
        ElementCss::new().line_color(color)
    }

    fn get_mmd_css(idx: usize) -> ElementStyle {
        let color = Self::get_color(idx);
        ElementStyle::new().line_color(color)
    }
}

impl ColorSet for BackgroundColor {
    const COLORS: &[Color] = BG_COLORS;

    fn get_class_name(idx: usize) -> String {
        format!("background-{}", idx % Self::COLORS.len())
    }

    fn get_uml_css(idx: usize) -> ElementCss {
        let color = Self::get_color(idx);
        ElementCss::new().background_color(color)
    }

    fn get_mmd_css(idx: usize) -> ElementStyle {
        let color = Self::get_color(idx);
        ElementStyle::new().background_color(color)
    }
}

impl ColorSet for BackGroundEdgeColor {
    const COLORS: &[Color] = BG_COLORS;

    fn get_class_name(idx: usize) -> String {
        format!("background-edge-{}", idx % Self::COLORS.len())
    }

    fn get_uml_css(idx: usize) -> ElementCss {
        let color = Self::get_color(idx);
        ElementCss::new().line_color(color).line_thickness(1.25)
    }

    fn get_mmd_css(idx: usize) -> ElementStyle {
        let color = Self::get_color(idx);
        ElementStyle::new()
            .background_color(color)
            .line_thickness(1.25)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Hash,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    scopegraphs::Label,
)]
#[derive(DeepSizeOf)]
pub enum SgLabel {
    Parent,
    Declaration,
    Method,
    Implement,
    Extend,
}

#[cfg(test)]
impl From<char> for SgLabel {
    fn from(c: char) -> Self {
        match c {
            'P' => Self::Parent,
            'D' => Self::Declaration,
            'M' => Self::Method,
            'I' => Self::Implement,
            'E' => Self::Extend,
            _ => panic!("Invalid SgLabel character: {}", c),
        }
    }
}

impl std::fmt::Display for SgLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char())
    }
}

impl ScopeGraphLabel for SgLabel {
    fn char(&self) -> char {
        match self {
            Self::Parent => 'P',
            Self::Declaration => 'D',
            Self::Method => 'M',
            Self::Implement => 'I',
            Self::Extend => 'E',
        }
    }

    fn str(&self) -> &'static str {
        match self {
            Self::Parent => "Parent",
            Self::Declaration => "Declaration",
            Self::Method => "Method",
            Self::Implement => "Implement",
            Self::Extend => "Extend",
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[derive(DeepSizeOf)]
pub enum SgData {
    #[default]
    NoData,
    Variable(Arc<str>, Arc<str>),
}

impl SgData {
    pub fn var(x: impl ToString, t: impl ToString) -> Self {
        Self::Variable(Arc::from(x.to_string()), Arc::from(t.to_string()))
    }

    pub fn name(&self) -> &str {
        match self {
            Self::NoData => "no data",
            Self::Variable(x, _) => x,
        }
    }
}

impl std::fmt::Display for SgData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoData => write!(f, ""),
            // Self::Variable(x, t) => write!(f, "{x}: {t}"),
            Self::Variable(x, t) => write!(f, "{x}: {t}"),
        }
    }
}

impl ScopeGraphData for SgData {
    fn variant_has_data(&self) -> bool {
        match self {
            Self::NoData => false,
            Self::Variable(_, _) => true,
        }
    }

    fn render_string(&self) -> String {
        format!("{}", self)
    }

    fn render_with_type(&self) -> String {
        match self {
            Self::NoData => String::new(),
            Self::Variable(name, ty) => format!("{name}: {ty}"),
        }
    }
}

pub type LibGraph<'a> = scopegraphs::ScopeGraph<'a, SgLabel, SgData, UncheckedCompleteness>;
pub type LibScope = scopegraphs::Scope;

impl RenderScopeData for SgData {
    fn render_node(&self) -> Option<String> {
        self.variant_has_data().then(|| self.render_string())
    }

    fn render_node_label(&self) -> Option<String> {
        None
    }

    fn extra_edges(&self) -> Vec<scopegraphs::render::EdgeTo> {
        Vec::new()
    }

    fn definition(&self) -> bool {
        self.render_node().is_some()
    }
}

impl RenderScopeLabel for SgLabel {
    fn render(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(DeepSizeOf)]
pub enum SgProjection {
    None,
    VarName,
    VarNameType,
}

impl ScopeGraphDataProjection<SgData> for SgProjection {
    type Output = Arc<str>;

    #[inline(always)]
    fn project(&self, data: &SgData) -> Self::Output
    where
        SgData: ScopeGraphData,
    {
        match self {
            Self::None => Arc::from(""),
            Self::VarName => Arc::from(data.name()),
            Self::VarNameType => Arc::from(data.render_string()),
        }
    }
}

impl std::fmt::Display for SgProjection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::VarName => write!(f, "var name"),
            Self::VarNameType => write!(f, "var name and type"),
        }
    }
}

#[macro_export]
macro_rules! debug_tracing {
    (trace, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        tracing::trace!($($arg)*);
    };

    (debug, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        tracing::debug!($($arg)*);
    };

    (info, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        tracing::info!($($arg)*);
    };
}