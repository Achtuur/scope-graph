mod item;
use std::{fs, io::{self, Write}, path::PathBuf, str::FromStr};

pub use item::*;
use theme::StyleSheet;

pub mod theme;

const HEADER_SECTION: &str = r#"
'skinparam linetype ortho

' this hides the <<class>> from nodes
hide stereotype"#;


pub struct PlantUmlDiagram {
    style: StyleSheet,
    items: Vec<PlantUmlItem>,
    title: String,
}

impl PlantUmlDiagram {
    pub fn new(title: impl ToString) -> Self {
        Self {
            style: StyleSheet::new(),
            items: Vec::new(),
            title: title.to_string(),
        }
    }

    pub fn set_style_sheet(&mut self, style: StyleSheet) {
        self.style = style;
    }

    pub fn push(&mut self, item: PlantUmlItem) {
        self.items.push(item);
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = PlantUmlItem>) {
        self.items.extend(items);
    }

    pub fn as_uml(&self) -> String {
        let css = &self.style.as_css();
        let header = format!("@startuml \"{}\"{}\n{}", self.title, HEADER_SECTION, css);
        let mut items = self.items.clone();
        items.sort();
        let body = items
            .iter()
            .map(|item| item.as_uml())
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}\n{}\n@enduml", header, body)
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), io::Error> {
        let path = PathBuf::from_str(path).unwrap();
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir)?;
        let content = self.as_uml();
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
