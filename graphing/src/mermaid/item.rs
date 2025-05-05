use std::sync::atomic::{AtomicUsize, Ordering};

use super::{MermaidStyleSheet, theme::EdgeType};

static EDGE_CTR: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub enum ItemShape {
    #[display("circle")]
    Circle,
    #[display("rounded")]
    Rounded,
    #[display("stadium")]
    Stadium,
    #[display("braces")]
    Braces,
    #[display("card")]
    Card,
}

pub struct MermaidNode {
    label: String,
    shape: ItemShape,
}

pub struct MermaidEdge {
    from: String,
    to: String,
    label: String,
    line_type: EdgeType,
}

pub enum MermaidItemKind {
    Node(MermaidNode),
    Edge(MermaidEdge),
}

impl MermaidItemKind {
    pub fn to_mmd(&self, id: &str) -> String {
        match self {
            MermaidItemKind::Node(node) => {
                format!(
                    "{}@{{ shape: {}, label: \"<span>{}</span>\" }};",
                    id, node.shape, node.label
                )
            }
            MermaidItemKind::Edge(edge) => {
                let line = match edge.label.as_str() {
                    "" => match edge.line_type {
                        EdgeType::Solid => "-->",
                        EdgeType::Dotted => "-.->",
                        EdgeType::Thick => "==>",
                    }
                    .to_string(),

                    lbl => match edge.line_type {
                        EdgeType::Solid => format!("-- {} -->", lbl),
                        EdgeType::Dotted => format!("-. {} .->", lbl),
                        EdgeType::Thick => format!("== {} ==>", lbl),
                    },
                };
                format!("{} {}@{} {};", edge.from, id, line, edge.to)
            }
        }
    }
}

pub struct MermaidItem {
    id: String,
    kind: MermaidItemKind,
    classes: Vec<String>,
}

impl MermaidItem {
    pub fn edge(
        from: impl ToString,
        to: impl ToString,
        label: impl ToString,
        line_type: EdgeType,
    ) -> Self {
        let num = EDGE_CTR.fetch_add(1, Ordering::Relaxed);
        Self {
            id: format!("edge{}", num),
            kind: MermaidItemKind::Edge(MermaidEdge {
                from: from.to_string(),
                to: to.to_string(),
                label: label.to_string(),
                line_type,
            }),
            classes: Vec::new(),
        }
    }

    pub fn node(id: impl ToString, label: impl ToString, shape: ItemShape) -> Self {
        Self {
            id: id.to_string(),
            kind: MermaidItemKind::Node(MermaidNode {
                label: label.to_string(),
                shape,
            }),
            classes: Vec::new(),
        }
    }

    pub fn add_class(mut self, class: impl ToString) -> Self {
        self.classes.push(class.to_string());
        self
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn find_nonexistant_class(&self, sheet: &MermaidStyleSheet) -> Option<&str> {
        self.classes.iter().find_map(|class| {
            if !sheet.contains_key(class) {
                Some(class.as_str())
            } else {
                None
            }
        })
    }

    pub(crate) fn to_mmd(&self) -> String {
        let item = self.kind.to_mmd(&self.id);
        let classes = self
            .classes
            .iter()
            .map(|class| format!("class {} {}", self.id, class))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}\n{}", item, classes)
    }
}
