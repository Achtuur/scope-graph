use std::io::{BufWriter, Write};

use crate::RenderResult;

pub trait Renderer {
    fn render_to_writer(&self, writer: &mut impl Write) -> RenderResult<()>;

    fn render(&self) -> RenderResult<String> {
        let mut buf = Vec::new();
        self.render_to_writer(&mut buf)?;
        String::from_utf8(buf).map_err(Into::into)
    }

    fn render_to_file(&self, path: &str) -> RenderResult<()> {
        let path = std::path::PathBuf::from(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        let mut buf = BufWriter::with_capacity(4096, file);
        self.render_to_writer(&mut buf)
    }
}