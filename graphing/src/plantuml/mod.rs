mod item;
use std::{cmp::Reverse, collections::BinaryHeap, io::Write};

pub use item::*;
use theme::PlantUmlStyleSheet;

use crate::{RenderResult, Renderer};

pub mod theme;

const HEADER_SECTION: &str = r#"
'skinparam linetype ortho

' this hides the <<class>> from nodes
hide stereotype"#;

#[derive(Clone, Debug)]
pub struct PlantUmlDiagram {
    style: PlantUmlStyleSheet,
    // notes have to come after nodes, so must be sorted
    items: BinaryHeap<Reverse<PlantUmlItem>>,
    title: String,
}

impl PlantUmlDiagram {
    pub fn new(title: impl ToString) -> Self {
        Self {
            style: PlantUmlStyleSheet::new(),
            items: BinaryHeap::new(),
            title: title.to_string(),
        }
    }

    pub fn set_title(&mut self, title: impl ToString) {
        self.title = title.to_string();
    }

    /// Returns number of items in the diagram.
    pub fn num_items(&self) -> usize {
        self.items.len()
    }

    pub fn set_style_sheet(&mut self, style: PlantUmlStyleSheet) {
        self.style = style;
    }

    pub fn push(&mut self, mut item: PlantUmlItem) {
        if let Some(class) = item.class_def() {
            self.style.push(class);
        }
        self.items.push(Reverse(item));
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = PlantUmlItem>) {
        for item in items {
            self.push(item);
        }
    }
}

impl Renderer for PlantUmlDiagram {
    fn render_to_writer(&self, writer: &mut impl Write) -> RenderResult<()> {
        writeln!(writer, "@startuml \"{}\"{}", self.title, HEADER_SECTION)?;
        // writes <style>...</style> section
        self.style.write(writer)?;
        let _ = writer.write(b"\n")?;
        let items = self.items.clone();
        for item in items {
            item.0.write(writer)?;
            let _ = writer.write(b"\n")?;
        }
        write!(writer, "\n@enduml")?;
        Ok(())
    }
}
