mod stlc;

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

fn main() {
    let storage = Storage::new();
    let sg = StlcGraph::new(&storage, ImplicitClose::default());
    let s0 = sg.add_scope_default();
    Expression::example_program_subtyping().expr_type(&sg, s0);

    sg.render_to("output.mmd", RenderSettings::default())
        .unwrap();
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
