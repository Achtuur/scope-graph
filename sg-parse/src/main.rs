mod stlc;

use std::path::PathBuf;
use std::str::FromStr;

use sclang::SclangExpression;
use scopegraphs::render::{EdgeStyle, EdgeTo};
use scopegraphs::{completeness::ImplicitClose, render::RenderSettings, Scope, Storage};
use stlc::*;

impl scopegraphs::render::RenderScopeLabel for StlcLabel {
    fn render(&self) -> String {
        self.to_string()
    }
}

impl scopegraphs::render::RenderScopeData for StlcData {
    fn render_node(&self) -> Option<String> {
        match self {
            StlcData::Variable(name, ty) => Some(format!("{name}: {ty}")),
            _ => None,
        }
    }

    fn render_node_label(&self) -> Option<String> {
        None
    }

    fn extra_edges(&self) -> Vec<scopegraphs::render::EdgeTo> {
        match self {
            StlcData::Variable(_, StlcType::Record(n)) => {
                let e = EdgeTo {
                    to: Scope(*n),
                    edge_style: EdgeStyle {},
                    label_text: "(Rec)".to_string(),
                };
                vec![e]
            }
            _ => Vec::with_capacity(0),
        }
    }

    fn definition(&self) -> bool {
        matches!(self, StlcData::Variable(_, _))
    }
}

const SRC_DIR: &str = "examples/";
const OUTPUT_DIR: &str = "output/";

macro_rules! path {
    ($dir: expr, $name:ident . $ext: ident) => {{
        let mut path = PathBuf::from_str($dir).unwrap().join(stringify!($name));
        path.set_extension(stringify!($ext));
        path
    }};
}

fn main() {
    println!("Initialising scope graph");
    let storage = Storage::new();
    let sg = StlcGraph::new(&storage, ImplicitClose::default());
    let s0 = sg.add_scope_default();

    let timer = std::time::Instant::now();

    let path = path!(SRC_DIR, first_example.sclang);
    let expr = match SclangExpression::from_file(&path) {
        Ok(expr) => expr,
        Err(e) => panic!("Error parsing {:?}: {}", path.as_path(), e),
    };
    println!("Parsing {:?} took {:?}", path.as_path(), timer.elapsed());

    let timer = std::time::Instant::now();
    SgExpression::new(&expr).expr_type(&sg, s0);
    println!("Creating scope graph took {:?}", timer.elapsed());

    println!("Rendering scope graph...");
    let path = path!(OUTPUT_DIR, first_example.mmd);
    sg.render_to(path, RenderSettings::default()).unwrap();
    println!("Done!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivalence() {
        let t1 = StlcData::Variable("x".to_string(), StlcType::Num);
        let t2 = StlcData::Variable("x".to_string(), StlcType::Bool);
        assert_ne!(t1, t2);

        let t3 = StlcData::Variable("x".to_string(), StlcType::Num);
        assert_eq!(t1, t3);

        let t4 = StlcData::Variable("y".to_string(), StlcType::Num);
        assert_ne!(t1, t4);
    }
}
