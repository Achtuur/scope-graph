mod item;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    str::FromStr,
};

pub use item::*;
use theme::PlantUmlStyleSheet;

pub mod theme;

const HEADER_SECTION: &str = r#"
'skinparam linetype ortho

' this hides the <<class>> from nodes
hide stereotype"#;

#[derive(Clone, Debug)]
pub struct PlantUmlDiagram {
    style: PlantUmlStyleSheet,
    items: Vec<PlantUmlItem>,
    title: String,
}

impl PlantUmlDiagram {
    pub fn new(title: impl ToString) -> Self {
        Self {
            style: PlantUmlStyleSheet::new(),
            items: Vec::new(),
            title: title.to_string(),
        }
    }

    pub fn set_style_sheet(&mut self, style: PlantUmlStyleSheet) {
        self.style = style;
    }

    pub fn push(&mut self, item: PlantUmlItem) {
        self.items.push(item);
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = PlantUmlItem>) {
        self.items.extend(items);
    }

    pub fn as_uml(mut self) -> String {
        self.items
            .iter_mut()
            .filter_map(|i| i.class_def())
            .for_each(|c| self.style.push(c));

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

    pub fn write_to_file(self, path: &str) -> Result<(), io::Error> {
        let mut path = PathBuf::from_str(path).unwrap();
        path.set_extension("puml");
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
