mod item;
pub use item::*;

mod theme;
pub use theme::*;

pub mod color;
pub use color::*;

pub struct PlantUmlDiagram<'a> {
    items: Vec<PlantUmlItem>,
    title: &'a str,
}

impl<'a> PlantUmlDiagram<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            items: Vec::new(),
            title,
        }
    }

    pub fn push(&mut self, item: PlantUmlItem) {
        self.items.push(item);
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = PlantUmlItem>) {
        self.items.extend(items);
    }

    pub fn as_uml(&self) -> String {
        let header = format!("@startuml \"{}\"\n'skinparam linetype ortho", self.title);
        let mut items = self.items.clone();
        items.sort();
        let body = items
        .iter()
        .map(|item| item.as_uml())
        .collect::<Vec<_>>()
        .join("\n");
        format!("{}\n{}\n@enduml", header, body)
    }
}