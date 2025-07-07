use std::{
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::RenderResult;

use super::{MermaidStyleSheet, sanitise_label, theme::EdgeType};

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
    pub fn write(&self, writer: &mut impl Write, id: &str) -> RenderResult<()> {
        match self {
            // {id}@{{ shape: {shape}, label: \"<span>{label}</span>\" }};
            MermaidItemKind::Node(node) => {
                write!(
                    writer,
                    "{}@{{ shape: {}, label: \"<span>{}</span>\" }};",
                    id, node.shape, node.label
                )?;
            }
            // {from} {id}@{line_type} {to};
            MermaidItemKind::Edge(edge) => {
                write!(writer, "{} {}@", edge.from, id)?;
                match edge.label.as_str() {
                    // no label (line length depends on number of dashes)
                    "" => match edge.line_type {
                        EdgeType::Solid => write!(writer, "-->"),
                        EdgeType::Dotted => write!(writer, "-.->"),
                        EdgeType::Thick => write!(writer, "==>"),
                    },
                    // with label
                    lbl => match edge.line_type {
                        EdgeType::Solid => write!(writer, "-- {} -->", lbl),
                        EdgeType::Dotted => write!(writer, "-. {} .->", lbl),
                        EdgeType::Thick => write!(writer, "== {} ==>", lbl),
                    },
                }?;
                write!(writer, " {};", edge.to)?;
            }
        }
        Ok(())
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
                label: sanitise_label(label),
                line_type,
            }),
            classes: Vec::new(),
        }
    }

    pub fn node(id: impl ToString, label: impl ToString, shape: ItemShape) -> Self {
        Self {
            id: id.to_string(),
            kind: MermaidItemKind::Node(MermaidNode {
                label: sanitise_label(label),
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

    pub(crate) fn write(&self, writer: &mut impl Write) -> RenderResult<()> {
        self.kind.write(writer, &self.id)?;
        let _ = writer.write(b"\n")?;
        for class in &self.classes {
            writeln!(writer, "class {} {}", self.id, class)?;
        }
        Ok(())
    }
}
