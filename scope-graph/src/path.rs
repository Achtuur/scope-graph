use std::{array::IntoIter, rc::Rc};

use crate::{label::ScopeGraphLabel, scope::Scope};

#[derive(Debug, Clone)]
pub enum Path<Lbl: ScopeGraphLabel> {
    Start(Scope),
    Step {
        label: Lbl,
        target: Scope,
        from: Box<Self>,
    },
}

impl <Lbl: ScopeGraphLabel> Path<Lbl> {
    pub fn start(start: Scope) -> Self {
        Self::Start(start)
    }

    pub fn step(self, label: Lbl, scope: Scope) -> Self {
        Self::Step {
            label,
            target: scope,
            from: Box::new(self),
        }
    }

    pub fn as_lbl_vec(&self) -> Vec<&Lbl> {
        let mut v = Vec::new();
        let mut current = self;
        while let Path::Step { from, label, .. } = current {
            v.push(label);
            current = from;
        }
        v
    }

    pub fn as_mmd(&self, mut mmd: String) -> String {
        match self {
            Self::Start(_) => mmd,
            Self::Step {from, label, target} => {
                mmd += "\n";
                mmd += &format!("scope_{} -.{}.-> scope_{}", from.scope_num(), label.char(), target.0);
                from.as_mmd(mmd)
            }
        }
    }

    /// Identical to using `std::fmt::Display`
    pub fn display(&self) -> String {
        match self {
            Self::Start(s) => format!("{s:?}"),
            Self::Step { from, label, target } => {
                format!("{} -{}-> {:?}", from.display(), label.char(), target)
            }
        }
    }

    fn scope_num(&self) -> usize {
        match self {
            Self::Start(s) => s.0,
            Self::Step { target, .. } => target.0,
        }
    }

}

impl<Lbl: ScopeGraphLabel> std::fmt::Display for Path<Lbl> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}