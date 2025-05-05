use std::sync::atomic::AtomicUsize;

use data::ScopeGraphData;
use graphing::{
    Color,
    mermaid::{MermaidStyleSheet, theme::ElementStyle},
    plantuml::theme::{ElementCss, PlantUmlStyleSheet},
};
use label::ScopeGraphLabel;
use serde::{Deserialize, Serialize};

pub mod label;
pub mod path;
pub mod scope;

pub mod data;
pub mod generator;
pub mod graph;
pub mod order;
pub mod regex;

/// Enable caching when doing forward resolution
pub const FORWARD_ENABLE_CACHING: bool = true;
/// Draw caches in the graph
pub const DRAW_CACHES: bool = false;
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

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SgLabel {
    Parent,
    Declaration,
    A,
    B,
    C,
    /// Debug path that should never be taken
    NeverTake,
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
            Self::NeverTake => 'W',
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
        }
    }

    fn str(&self) -> &'static str {
        match self {
            Self::Parent => "Parent",
            Self::Declaration => "Declaration",
            Self::NeverTake => "NeverTake",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SgData {
    NoData,
    Variable(String, String),
}

impl SgData {
    pub fn var(x: impl ToString, t: impl ToString) -> Self {
        Self::Variable(x.to_string(), t.to_string())
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
            Self::NoData => write!(f, "no data"),
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
        match self {
            Self::NoData => "".to_string(),
            Self::Variable(x, t) => format!("{}: {}", x, t),
        }
    }
}
