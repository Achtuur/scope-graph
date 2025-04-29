use std::sync::atomic::{AtomicUsize, Ordering};

use super::theme::LineType;

static EDGE_CTR: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub enum ItemShape {
    #[display("circle")]
    Circle,
    #[display("rounded")]
    Rounded,
    #[display("stadium")]
    Stadium
}


pub struct MermaidNode {
    id: String,
    label: String,
    shape: ItemShape,
}

pub struct MermaidEdge {
    from: String,
    to: String,
    label: String,
    line_type: LineType,
    animated: bool,
}

pub enum MermaidItem {
    Node(MermaidNode),
    Edge(MermaidEdge),
}

impl MermaidItem {
    pub fn to_mmd(&self) -> String {
        match self {
            MermaidItem::Node(node) => {
                format!("{}@{{ shape: {}, label: \"{}\" }}", node.id, node.shape, node.label)
            },
            MermaidItem::Edge(edge) => {
                let line = match edge.label.as_str() {
                    "" => match edge.line_type {
                        LineType::Solid => "-->",
                        LineType::Dotted => "-.->",
                        LineType::Thick => "==>",
                    }.to_string(),

                    lbl => match edge.line_type {
                        LineType::Solid => format!("--{}-->", lbl),
                        LineType::Dotted => format!("-.{}.->", lbl),
                        LineType::Thick => format!("== {} ==>", lbl),
                    }
                };
                let id = format!("edge{}", EDGE_CTR.fetch_add(1, Ordering::Relaxed));
                format!("{} {}@{} {}", edge.from, id, line, edge.to)
            },
        }
    }
}