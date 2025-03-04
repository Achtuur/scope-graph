use crate::{label::ScopeGraphLabel, scope::Scope};

/// Path enum "starts" at the target scope, ie its in reverse order
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path<Lbl>
where Lbl: ScopeGraphLabel + Clone
{
    Start(Scope),
    Step {
        label: Lbl,
        target: Scope,
        from: Box<Self>,
    },
}

impl<Lbl: ScopeGraphLabel> Path<Lbl>
where Lbl: ScopeGraphLabel + Clone
{
    pub fn start(start: Scope) -> Self {
        Self::Start(start)
    }

    /// Step forward (p -> new p)
    pub fn step(self, label: Lbl, scope: Scope) -> Self {
        Self::Step {
            label,
            target: scope,
            from: Box::new(self),
        }
    }

    // step backwards (new p -> p)
    pub fn step_back(self, label: Lbl, scope: Scope) -> Self {
        match self {
            Self::Start(s) => {
                Self::Step {
                    label,
                    target: s,
                    from: Box::new(Self::start(scope)),
                }
            }

            Self::Step { label: lbl, target: tgt, from } => {
                Self::Step {
                    label: lbl,
                    target: tgt,
                    from: Box::new(from.step_back(label, scope)),
                }
            }
        }
    }

    pub fn target(&self) -> Scope {
        match self {
            Self::Start(s) => *s,
            Self::Step { target, .. } => *target,
        }
    }

    pub fn as_lbl_vec(&self) -> Vec<&Lbl> {
        let mut v = Vec::new();
        let mut current = self;
        while let Path::Step { from, label, .. } = current {
            v.push(label);
            current = from;
        }
        v.reverse();
        v
    }


    pub fn append(&mut self, other: Self) {
        // keep going deeper into from, start

        // self must end with a step to scope S
        // other must have a start scope S

        match self {
            Self::Start(s) => {
                match other {
                    Self::Step { label, target, from } if target == *s => {
                        *self = Self::Step {
                            label,
                            target,
                            from,
                        }
                    }
                    _ => panic!("unmergable paths"),
                }
            }
            Self::Step { label, target, from } => {
                from.append(other);
            }
        }
    }

    pub fn trim_matching_start(self, other: &Self) -> Self {
        match (self, other) {
            (Self::Step { target, label, from }, Self::Step { target: target2, ..}) => {
                // cut off the rest of the path
                if &target == target2 {
                    Self::Start(target)
                } else {
                    Self::Step {
                        label,
                        target,
                        from: Box::new(from.trim_matching_start(other)),
                    }
                }
            }

            (x, _) => x,
        }
    }

    // pub fn trim_start_matching(self, other: &Self) -> Self {
    //     let mut current = self;
    //     let mut other = other;
    //     loop {
    //         match (current, other) {
    //             (Self::Start(s), _) => {
    //                 Self::Start(s)
    //             }


    //             _ => current
    //         }
    //     }
    // }

    pub fn as_mmd_debug(&self, mut mmd: String) -> String {
        match self {
            Self::Start(_) => mmd,
            Self::Step {
                from,
                label,
                target,
            } => {
                mmd += "\n";
                mmd += &format!(
                    "scope_{} -..-> scope_{}",
                    from.scope_num(),
                    // label.char(),
                    target.0
                );
                from.as_mmd_debug(mmd)
            }
        }
    }

    pub fn as_mmd(&self, mut mmd: String) -> String {
        match self {
            Self::Start(_) => mmd,
            Self::Step {
                from,
                label,
                target,
            } => {
                mmd += "\n";
                mmd += &format!(
                    "scope_{} --{}--> scope_{}",
                    from.scope_num(),
                    label.char(),
                    target.0
                );
                from.as_mmd(mmd)
            }
        }
    }

    /// Identical to using `std::fmt::Display`
    pub fn display(&self) -> String {
        match self {
            Self::Start(s) => format!("{s:?}"),
            Self::Step {
                from,
                label,
                target,
            } => {
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

impl<Lbl> std::fmt::Display for Path<Lbl>
where Lbl: ScopeGraphLabel + Clone{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}
