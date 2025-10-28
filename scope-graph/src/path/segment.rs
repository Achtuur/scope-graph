use crate::{label::ScopeGraphLabel, path::Path, scope::Scope};

/// One part of a path
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PathSegment<'a, Lbl> {
    Start(&'a Scope),
    Step {
        automaton_idx: &'a usize,
        label: &'a Lbl,
        target: &'a Scope,
        from: &'a Scope,
        len: usize,
    },
}

impl<Lbl: ScopeGraphLabel> std::fmt::Display for PathSegment<'_, Lbl> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Start(s) => write!(f, "{s}"),
            Self::Step {
                automaton_idx,
                label,
                target,
                from,
                ..
            } => {
                write!(
                    f,
                    "{} -{}{}-> {}",
                    from,
                    label.char(),
                    automaton_idx,
                    target
                )
            }
        }
    }
}

impl<'a, Lbl: ScopeGraphLabel> PathSegment<'a, Lbl> {
    #[inline]
    fn new(path: &'a Path<Lbl>) -> Self {
        match path {
            Path::Start(s) => Self::Start(s),
            Path::Step {
                automaton_idx,
                label,
                target,
                from,
                len,
                ..
            } => Self::Step {
                automaton_idx,
                label,
                target,
                from: from.target_ref(),
                len: *len,
            },
        }
    }

    #[inline]
    pub fn from_path(path: &'a Path<Lbl>) -> impl Iterator<Item = Self> {
        path.iter().map(Self::new)
    }

    pub fn from_path_with_offset(
        path: &'a Path<Lbl>,
        offset: usize,
        len: usize,
    ) -> impl Iterator<Item = Self> {
        path.iter().skip(offset).take(len).map(Self::new)
    }

    pub fn source(&self) -> &Scope {
        match self {
            Self::Start(s) => s,
            Self::Step { from, .. } => from,
        }
    }

    pub fn target(&self) -> &Scope {
        match self {
            Self::Start(s) => s,
            Self::Step { target, .. } => target,
        }
    }

    pub fn automaton_idx(&self) -> usize {
        match self {
            Self::Start(_) => 0,
            Self::Step { automaton_idx, .. } => **automaton_idx,
        }
    }

    pub fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Start(s1), Self::Start(s2)) => s1 == s2,
            (Self::Start(s), Self::Step { target: t, .. })
            | (Self::Step { target: t, .. }, Self::Start(s)) => {
                // false
                s == t
            }
            (
                Self::Step {
                    automaton_idx: a1,
                    label: l1,
                    target: t1,
                    from: f1,
                    ..
                },
                Self::Step {
                    automaton_idx: a2,
                    label: l2,
                    target: t2,
                    from: f2,
                    ..
                },
            ) => a1 == a2 && t1 == t2 && l1 == l2 && f1 == f2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path() {
        let p1: Path<char> = Path::start(1);
        let s1 = PathSegment::from_path(&p1).collect::<Vec<_>>();
        for s in s1 {
            println!("s: {}", s);
        }

        let p2: Path<char> = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        let s2 = PathSegment::from_path(&p2).collect::<Vec<_>>();
        for s in s2 {
            println!("s: {}", s);
        }
    }
}
