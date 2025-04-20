use crate::color::Color;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    fn uml_str(&self) -> &'static str {
        match self {
            EdgeDirection::Left => "l",
            EdgeDirection::Right => "r",
            EdgeDirection::Up => "u",
            EdgeDirection::Bottom => "b",
            EdgeDirection::Unspecified => "",
            EdgeDirection::Norank => "[norank]",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    Bold,
}

impl LineStyle {
    fn uml_str(&self) -> &'static str {
        match self {
            LineStyle::Solid => "",
            LineStyle::Dashed => "line.dashed",
            LineStyle::Dotted => "line.dotted",
            LineStyle::Bold => "line.bold",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
pub struct ItemAnnotation {
    line_style: Option<LineStyle>,
    text_color: Option<Color>,
    line_color: Option<Color>,
}

impl ItemAnnotation {
    fn is_default(&self) -> bool {
        self.line_style.is_none() && self.text_color.is_none() && self.line_color.is_none()
    }

    pub fn as_uml(&self) -> String {
        if self.is_default() {
            return String::new();
        }

        let s = [
            self.line_style.unwrap_or_default().uml_str().to_string(),
            format!("text:{}", self.text_color.unwrap_or_default()),
            format!("line:{}", self.line_color.unwrap_or_default()),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>()
        .join(";");
        format!("#{}", s.trim_end())
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
    }
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

    fn as_uml(&self, annotation_str: &str) -> String {
        match self {
            PlantUmlItemKind::Node { id, contents, node_type } => {
                format!("{} \"{}\" as {} {}", node_type.uml_str(), contents, id, annotation_str)
            },
            PlantUmlItemKind::Edge { from, to, label, dir,} => {
                format!("{} -{}-> {} {} : {}", from, dir.uml_str(), to, annotation_str, label)
            },
            PlantUmlItemKind::Note { to, contents } => {
                let formatted = contents.replace("\n", "\n\t");
                let note_key = format!("N_{0:}", to);
                let note = format!("note as {0:}\n\t{1:}\nend note", note_key, formatted);
                format!("{note}\n{0:} .r. {1:}", note_key, to)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlantUmlItem {
    item: PlantUmlItemKind,
    annotation: ItemAnnotation,
}

impl PlantUmlItem {
    pub fn new(item: PlantUmlItemKind) -> Self {
        Self {
            item,
            annotation: ItemAnnotation::default(),
        }
    }

    pub fn node(id: impl ToString, contents: impl ToString, node_type: NodeType) -> Self {
        Self::new(PlantUmlItemKind::Node {
            id: id.to_string(),
            contents: contents.to_string(),
            node_type,
        })
    }

    pub fn edge(from: impl ToString, to: impl ToString, label: impl ToString, dir: EdgeDirection) -> Self {
        Self::new(PlantUmlItemKind::Edge {
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
            dir,
        })
    }

    pub fn note(to: impl ToString, contents: impl ToString) -> Self {
        Self::new(PlantUmlItemKind::Note {
            to: to.to_string(),
            contents: contents.to_string(),
        })
    }

    pub fn with_line_style(mut self, style: LineStyle) -> Self {
        self.annotation.line_style = Some(style);
        self
    }

    pub fn with_text_color(mut self, color: crate::Color) -> Self {
        self.annotation.text_color = Some(color);
        self
    }

    pub fn with_line_color(mut self, line_color: Color) -> Self {
        self.annotation.line_color = Some(line_color);
        self
    }

    pub fn as_uml(&self) -> String {
        let annotation_str = self.annotation.as_uml();
        let s = self.item.as_uml(&annotation_str);
        s.trim_end().to_string()
    }
}
