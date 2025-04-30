use std::{collections::HashMap, fs, io::Write, ops::Deref, path::PathBuf, str::FromStr};

use item::{MermaidItem, MermaidNode};
use theme::ElementStyle;

pub mod item;
pub mod theme;

#[derive(Default, Debug)]
pub struct MermaidStyleSheet {
    map: HashMap<String, ElementStyle>,
}

impl MermaidStyleSheet {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn with_class(mut self, class: impl ToString, style: ElementStyle) -> Self {
        self.map.insert(class.to_string(), style);
        self
    }

    pub fn merge(&mut self, other: Self) {
        for (class, style) in other.map {
            self.map.insert(class, style);
        }
    }
}

impl FromIterator<(String, ElementStyle)> for MermaidStyleSheet {
    fn from_iter<T: IntoIterator<Item = (String, ElementStyle)>>(iter: T) -> Self {
        let mut map = HashMap::new();
        for (class, style) in iter {
            map.insert(class, style);
        }
        Self { map }
    }
}

impl Deref for MermaidStyleSheet {
    type Target = HashMap<String, ElementStyle>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

#[derive(derive_more::Display, Debug, Clone, Copy)]
pub enum MermaidChartDirection {
    #[display("TB")]
    TopBottom,
    #[display("BT")]
    BottomTop,
    #[display("LR")]
    LeftRight,
    #[display("RL")]
    RightLeft,
}

pub struct MermaidDiagram {
    style: MermaidStyleSheet,
    items: Vec<MermaidItem>,
    title: String,
    direction: MermaidChartDirection,
}

impl MermaidDiagram {
    pub fn new(title: impl ToString) -> Self {
        Self {
            style: MermaidStyleSheet::default(),
            items: Vec::new(),
            title: title.to_string(),
            direction: MermaidChartDirection::TopBottom,
        }
    }

    pub fn set_direction(&mut self, direction: MermaidChartDirection) {
        self.direction = direction;
    }

    pub fn set_style_sheet(&mut self, style: MermaidStyleSheet) {
        self.style = style;
    }

    pub fn push(&mut self, item: MermaidItem) {
        self.items.push(item);
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = MermaidItem>) {
        self.items.extend(items);
    }

    pub fn as_mmd(&self) -> String {
       // iterate over items and render them
       // pass along the stylesheet so they can select their class
        let classes = (*self.style).iter().map(|(class, style)| {
            style.as_classdef(class)
        })
        .collect::<Vec<_>>()
        .join("\n");

        for item in &self.items {
            if let Some(dne_class) = item.find_nonexistant_class(&self.style) {
                tracing::warn!("Class {} does not exist in the stylesheet (found in {})", dne_class, item.id());
            }
        }


        let body = self.items.iter().map(|item| {
            item.to_mmd()
        })
        .collect::<Vec<_>>()
        .join("\n");


        format!(
            "```mermaid\n\
            ---\n\
            title: \"{}\"\n\
            ---\n\
            flowchart {}\n\
            {}\n\
            {}\n\
            ```",
            self.title, self.direction, classes, body
        )
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        let mut path = PathBuf::from_str(path).unwrap();
        path.set_extension("md");
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir)?;
        let content = self.as_mmd();
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap();
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}