use plantuml::{Color, EdgeDirection, LineStyle, PlantUmlItem};

use crate::{label::ScopeGraphLabel, scope::Scope};

/// Path enum "starts" at the target scope, ie its in reverse order
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    Start(Scope),
    Step {
        label: Lbl,
        target: Scope,
        from: Box<Self>,
    },
}

impl<Lbl: ScopeGraphLabel> Path<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
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
            Self::Start(s) => Self::Step {
                label,
                target: s,
                from: Box::new(Self::start(scope)),
            },

            Self::Step {
                label: lbl,
                target: tgt,
                from,
            } => Self::Step {
                label: lbl,
                target: tgt,
                from: Box::new(from.step_back(label, scope)),
            },
        }
    }

    pub fn without_target(&self) -> Option<&Self> {
        match self {
            Self::Start(_) => None,
            Self::Step { from, .. } => Some(from),
        }
    }

    pub fn prepend(self, other: &Self) -> Self {
        match (self, other) {
            (
                Self::Start(s),
                Self::Step {
                    label,
                    target,
                    from,
                },
            ) if target.0 == s.0 => Self::Step {
                label: label.clone(),
                target: *target,
                from: from.clone(),
            },
            (
                Self::Step {
                    label,
                    target,
                    from,
                },
                o @ Self::Step { .. },
            ) => Self::Step {
                label,
                target,
                from: Box::new(from.prepend(o)),
            },
            // rhs is Start here
            (p, _) => {
                // prepending start just results in self
                p
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

    pub fn extend(&mut self, extension: Self) {
        let mut current = self;
        while let Path::Step { from, .. } = current {
            current = from;
        }
        *current = extension;
    }

    pub fn append(&mut self, other: Self) {
        // keep going deeper into from, start

        // self must end with a step to scope S
        // other must have a start scope S

        match self {
            Self::Start(s) => match other {
                Self::Step {
                    label,
                    target,
                    from,
                } if target == *s => {
                    *self = Self::Step {
                        label,
                        target,
                        from,
                    }
                }
                _ => panic!("unmergable paths"),
            },
            Self::Step {
                label,
                target,
                from,
            } => {
                from.append(other);
            }
        }
    }

    pub fn trim_matching_start(self, other: &Self) -> Self {
        match (self, other) {
            (
                Self::Step {
                    target,
                    label,
                    from,
                },
                Self::Step {
                    target: target2, ..
                },
            ) => {
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

    pub fn mem_size(&self) -> usize {
        match self {
            Self::Start(_) => std::mem::size_of::<Self>(),
            Self::Step { from, .. } => std::mem::size_of::<Self>() + from.mem_size(),
        }
    }

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

    /// Transforms path to uml arrows. This can be multiple lines.
    ///
    /// # Arguments
    ///
    /// * `color` - The color of the arrow
    /// * `reverse` - If true, the arrow will be reversed
    pub fn as_uml<'a>(&self, color: Color, reverse: bool) -> Vec<PlantUmlItem> {
        match self {
            Self::Start(_) => Vec::new(),
            Self::Step {
                from,
                label,
                target,
            } => {
                let (from_scope, to_scope) = match reverse {
                    false => (from.scope(), target),
                    true => (target, from.scope()),
                };

                let item = PlantUmlItem::edge(
                    from_scope.uml_id(),
                    to_scope.uml_id(),
                    label.char(),
                    EdgeDirection::Norank,
                )
                .with_line_color(color)
                .with_line_style(LineStyle::Dashed);

                let mut from_items = from.as_uml(color, reverse);
                from_items.push(item);
                from_items
            }
        }
    }

    /// Identical to using `std::fmt::Display`
    pub fn display(&self) -> String {
        match self {
            Self::Start(s) => format!("{}", s),
            Self::Step {
                from,
                label,
                target,
            } => {
                format!("{} -{}-> {}", from.display(), label.char(), target)
            }
        }
    }

    fn scope_num(&self) -> usize {
        match self {
            Self::Start(s) => s.0,
            Self::Step { target, .. } => target.0,
        }
    }

    fn scope(&self) -> &Scope {
        match self {
            Self::Start(s) => s,
            Self::Step { target, .. } => target,
        }
    }
}

impl<Lbl> std::fmt::Display for Path<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}
