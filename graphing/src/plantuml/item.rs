use std::{
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{Color, RenderResult};

use super::theme::{CssClass, ElementCss, LineStyle};

static CLASS_CTR: AtomicUsize = AtomicUsize::new(0);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeDirection {
    #[default]
    Unspecified,
    Left,
    Right,
    Up,
    Bottom,
    Norank,
}

impl EdgeDirection {
    fn edge_str(&self) -> &'static str {
        match self {
            EdgeDirection::Left => "l",
            EdgeDirection::Right => "r",
            EdgeDirection::Up => "u",
            EdgeDirection::Bottom => "d",
            EdgeDirection::Unspecified => "",
            EdgeDirection::Norank => "[norank]",
        }
    }

    fn note_str(&self) -> &'static str {
        match self {
            EdgeDirection::Left => "left",
            EdgeDirection::Right => "right",
            EdgeDirection::Up => "top",
            EdgeDirection::Bottom => "bottom",
            _ => "left", // default to left
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodeType {
    /// Node, used for scopes
    Node,
    /// Card, used for declarations
    Card,
}

impl NodeType {
    pub fn uml_str(&self) -> &'static str {
        match self {
            NodeType::Node => "usecase",
            NodeType::Card => "card",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItemAnnotation {
    line_style: Option<LineStyle>,
    text_color: Option<Color>,
    line_color: Option<Color>,
}

impl From<ItemAnnotation> for ElementCss {
    fn from(value: ItemAnnotation) -> Self {
        let mut el = ElementCss::new();
        if let Some(x) = value.line_style {
            el = el.line_style(x);
        }
        if let Some(x) = value.text_color {
            el = el.font_color(x);
        }
        if let Some(x) = value.line_color {
            el = el.line_color(x);
        }
        el
    }
}

impl ItemAnnotation {
    fn is_default(&self) -> bool {
        self.line_style.is_none() && self.text_color.is_none() && self.line_color.is_none()
    }

    pub fn as_css(&self) -> ElementCss {
        ElementCss::from(*self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlantUmlItemKind {
    Node {
        id: String,
        contents: String,
        node_type: NodeType,
    },
    Edge {
        from: String,
        to: String,
        label: String,
        dir: EdgeDirection,
    },
    Note {
        to: String,
        contents: String,
        dir: EdgeDirection,
    },
}

impl Ord for PlantUmlItemKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for PlantUmlItemKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.num().cmp(&other.num()))
    }
}

impl PlantUmlItemKind {
    fn num(&self) -> usize {
        match self {
            Self::Node { .. } => 0,
            Self::Edge { .. } => 1,
            Self::Note { .. } => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantUmlItem {
    kind: PlantUmlItemKind,
    classes: Vec<String>,
    annotation: ItemAnnotation,
}

impl PartialOrd for PlantUmlItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.kind.cmp(&other.kind))
    }
}

impl Ord for PlantUmlItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PlantUmlItem {
    pub fn new(item: PlantUmlItemKind) -> Self {
        Self {
            kind: item,
            classes: Vec::new(),
            annotation: ItemAnnotation::default(),
        }
    }

    /// Returns the ID of the node this item represents or is connected to.
    ///
    /// Edges use their 'from' node
    pub fn node_id(&self) -> &str {
        match &self.kind {
            PlantUmlItemKind::Node { id, .. } => id,
            PlantUmlItemKind::Edge { from, .. } => from,
            PlantUmlItemKind::Note { to, .. } => to,
        }
    }

    pub fn set_direction(&mut self, new_dir: EdgeDirection) {
        match &mut self.kind {
            PlantUmlItemKind::Node { .. } => (),
            PlantUmlItemKind::Edge { dir, .. } => *dir = new_dir,
            PlantUmlItemKind::Note { dir, .. } => *dir = new_dir,
        }
    }

    fn sanitise_id(id: impl ToString) -> String {
        id.to_string().chars().fold(String::new(), |mut s, c| {
            match c {
                _ if c.is_ascii_alphanumeric() => s.push(c),
                '_' => s.push(c),
                _ => (),
            }
            s
        })
    }

    pub fn node(id: impl ToString, contents: impl ToString, node_type: NodeType) -> Self {
        Self::new(PlantUmlItemKind::Node {
            id: Self::sanitise_id(id),
            contents: contents.to_string(),
            node_type,
        })
    }

    pub fn edge(
        from: impl ToString,
        to: impl ToString,
        label: impl ToString,
        dir: EdgeDirection,
    ) -> Self {
        Self::new(PlantUmlItemKind::Edge {
            from: Self::sanitise_id(from),
            to: Self::sanitise_id(to),
            label: label.to_string(),
            dir,
        })
    }

    pub fn note(to: impl ToString, contents: impl ToString, dir: EdgeDirection) -> Self {
        Self::new(PlantUmlItemKind::Note {
            to: Self::sanitise_id(to),
            contents: contents.to_string(),
            dir,
        })
    }

    pub fn add_class(mut self, class: impl ToString) -> Self {
        self.classes.push(class.to_string());
        self
    }

    pub fn with_line_style(mut self, style: LineStyle) -> Self {
        self.annotation.line_style = Some(style);
        self
    }

    pub fn with_text_color(mut self, color: Color) -> Self {
        self.annotation.text_color = Some(color);
        self
    }

    pub fn with_line_color(mut self, line_color: Color) -> Self {
        self.annotation.line_color = Some(line_color);
        self
    }

    /// Returns a CssClass if this object was not given a class and contains annotations
    pub(crate) fn class_def(&mut self) -> Option<CssClass> {
        if self.annotation.is_default() {
            return None;
        }

        let class_name = format!("gen-class-{}", CLASS_CTR.fetch_add(1, Ordering::Relaxed));
        self.classes.push(class_name.clone());
        let el = self.annotation.into();
        let class = CssClass::new_class(class_name, el);
        Some(class)
    }

    // pub fn as_uml(&self) -> String {
    //     let class = self
    //         .classes
    //         .iter()
    //         .map(|c| format!("<<{}>>", c))
    //         .collect::<Vec<_>>()
    //         .join("");
    //     let s = self.kind.as_uml(&class);
    //     s.trim_end().to_string()
    // }

    pub fn write(&self, writer: &mut impl Write) -> RenderResult<()> {
        match &self.kind {
            // {node_type} "{contents}" as {id} <<{classes}>>
            PlantUmlItemKind::Node {
                id,
                contents,
                node_type,
            } => {
                write!(writer, "{} \"{}\" as {}", node_type.uml_str(), contents, id,)?;
                self.write_class(writer)?;
            }
            // {from} -{dir}-> {to} {classes} : {label}
            PlantUmlItemKind::Edge {
                from,
                to,
                label,
                dir,
            } => {
                write!(writer, "{} -{}-> {}", from, dir.edge_str(), to)?;
                self.write_class(writer)?;
                if !label.is_empty() {
                    write!(writer, " : {label}")?;
                }
            }
            // note left of {to} {classes}\n\t{contents}\nend note
            PlantUmlItemKind::Note { to, contents, dir } => {
                match to.is_empty() {
                    true => write!(writer, "note {}", dir.note_str())?,
                    false => write!(writer, "note {} of {}", dir.note_str(), to)?,
                }
                self.write_class(writer)?;
                let formatted = contents.replace("\n", "\n\t");
                write!(writer, "\n\t\"{formatted}\"")?;
                write!(writer, "\nend note")?;
            }
        }
        Ok(())
    }

    fn write_class(&self, writer: &mut impl Write) -> RenderResult<()> {
        for class in &self.classes {
            write!(writer, "<<{class}>>")?;
        }
        Ok(())
    }
}
