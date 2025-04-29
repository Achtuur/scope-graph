use item::MermaidNode;

pub mod item;
pub mod theme;

pub struct MermaidDiagram {
    items: Vec<MermaidNode>,
    title: String,
}