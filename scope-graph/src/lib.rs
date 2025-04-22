use std::sync::atomic::AtomicUsize;

use plantuml::theme::Color;

pub mod label;
pub mod path;
pub mod scope;

pub mod data;
pub mod graph;
pub mod order;
pub mod regex;
pub mod resolve;

pub const COLORS: &[Color] = &[
    Color::RED,
    Color::GREEN,
    Color::PURPLE,
    Color::BLUE,
    Color::ORANGE,
];

pub static COLOR_POINTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_color() -> Color {
    let idx = COLOR_POINTER.load(std::sync::atomic::Ordering::Relaxed);
    let _ = COLOR_POINTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    COLORS[idx % COLORS.len()]
}

pub fn get_color(idx: usize) -> Color {
    COLORS[idx % COLORS.len()]
}

/// Enable caching when doing forward resolution
pub const FORWARD_ENABLE_CACHING: bool = true;

pub const DRAW_CACHES: bool = true;
