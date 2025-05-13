use std::{collections::HashSet, rc::Rc};

use graphing::{
    mermaid::{item::MermaidItem, theme::EdgeType},
    plantuml::{EdgeDirection, PlantUmlItem},
};

use crate::{label::ScopeGraphLabel, scope::Scope};

/// Path enum "starts" at the target scope, ie its in reverse order
///
/// This holds a path using a pointer to the head path segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    Start(Scope),
    Step {
        automaton_idx: usize,
        label: Lbl,
        target: Scope,
        from: Rc<Self>,
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
    pub fn step(&self, label: Lbl, scope: Scope, automaton_idx: usize) -> Self {
        Self::Step {
            label,
            target: scope,
            from: Rc::new(self.clone()),
            automaton_idx,
        }
    }

    pub fn target(&self) -> Scope {
        match self {
            Self::Start(s) => *s,
            Self::Step { target, .. } => *target,
        }
    }

    pub fn start_scope(&self) -> Scope {
        match self {
            Self::Start(s) => *s,
            Self::Step { from, .. } => from.start_scope(),
        }
    }

    pub fn is_circular(&self, prev_index: usize) -> bool {
        let mut current = self;
        let mut visited = HashSet::new();
        let mut prev_index = prev_index;
        loop {
            match current {
                Self::Start(s) => return !visited.insert((s, prev_index)),
                Self::Step {
                    target,
                    from,
                    automaton_idx,
                    ..
                } => {
                    if !visited.insert((target, *automaton_idx)) {
                        return true;
                    }
                    current = from;
                    prev_index = *automaton_idx;
                }
            }
        }
    }

    pub fn as_mmd(&self, class: String, reverse: bool) -> Vec<MermaidItem> {
        match self {
            Self::Start(_) => Vec::new(),
            Self::Step { from, target, .. } => {
                let (from_scope, to_scope) = match reverse {
                    false => (from.target(), *target),
                    true => (*target, from.target()),
                };
                let item = MermaidItem::edge(
                    from_scope.uml_id(),
                    to_scope.uml_id(),
                    // label.char(),
                    "",
                    EdgeType::Dotted,
                )
                .add_class(class.clone());

                let mut from_items = from.as_mmd(class, reverse);
                from_items.push(item);
                from_items
            }
        }
    }

    /// Transforms path to uml arrows. This can be multiple lines.
    ///
    /// # Arguments
    ///
    /// * `color` - The color of the arrow
    /// * `reverse` - If true, the arrow will be reversed
    pub fn as_uml(&self, class: String, reverse: bool) -> Vec<PlantUmlItem> {
        match self {
            Self::Start(_) => Vec::new(),
            Self::Step {
                from,
                label,
                target,
                ..
            } => {
                let (from_scope, to_scope) = match reverse {
                    false => (from.target(), *target),
                    true => (*target, from.target()),
                };

                let item = PlantUmlItem::edge(
                    from_scope.uml_id(),
                    to_scope.uml_id(),
                    label.char(),
                    EdgeDirection::Norank,
                )
                .add_class(class.clone())
                .add_class("query-edge");

                let mut from_items = from.as_uml(class, reverse);
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
                ..
            } => {
                format!("{} -{}-> {}", from.display(), label.char(), target)
            }
        }
    }

    pub fn display_with_mem_addr(&self) -> String {
        match self {
            Self::Start(s) => format!("{}", s),
            Self::Step {
                from,
                label,
                target,
                ..
            } => {
                let addr = Rc::as_ptr(from);
                format!(
                    "{} -{}-> {} ({:?})",
                    from.display_with_mem_addr(),
                    label.char(),
                    target,
                    addr
                )
            }
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

/// Path enum "starts" at the target scope, ie its in reverse order
///
/// Compared to `Path`, this is stored in reverse.
/// The pointer refers to the tail segment instead.
/// This is more efficient for the cache
///
/// Internally, this is the exact same structure, however the "start scope" now refers to the tail instead
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReversePath<Lbl>(Path<Lbl>)
where
    Lbl: ScopeGraphLabel + Clone;

impl<Lbl> From<Path<Lbl>> for ReversePath<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    fn from(value: Path<Lbl>) -> Self {
        let rev_path = match value {
            p @ Path::Start(_) => p,
            Path::Step {
                label,
                target,
                from,
                automaton_idx,
            } => {
                // what we have: from -L> target
                // what we want: target <L- from

                let mut rp = Path::Start(target);
                rp = rp.step(label, from.target(), automaton_idx);
                let mut current = from.as_ref();
                while let Path::Step { label, from, .. } = current {
                    rp = rp.step(label.clone(), from.target(), automaton_idx);
                    current = from;
                }
                rp
            }
        };
        ReversePath(rev_path)
    }
}

impl<Lbl> From<&Path<Lbl>> for ReversePath<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    fn from(value: &Path<Lbl>) -> Self {
        value.clone().into()
    }
}

impl<Lbl> AsRef<Path<Lbl>> for ReversePath<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    fn as_ref(&self) -> &Path<Lbl> {
        &self.0
    }
}

impl<Lbl> std::fmt::Display for ReversePath<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl<Lbl> ReversePath<Lbl>
where
    Lbl: ScopeGraphLabel + Clone,
{
    pub fn start(scope: Scope) -> Self {
        Self(Path::start(scope))
    }

    /// Gets the target of the current path.
    ///
    /// This is equal to Path::start_scope()
    pub fn target(&self) -> Scope {
        self.0.start_scope()
    }

    /// Gets the start scope of the current path.
    ///
    /// This is equal to Path::target()
    pub fn start_scope(&self) -> Scope {
        self.0.target()
    }

    /// Step forward (p -> new p)
    pub fn step(&self, label: Lbl, scope: Scope, automaton_idx: usize) -> Self {
        Self(self.0.step(label, scope, automaton_idx))
    }

    pub fn as_uml(&self, class: String, reverse: bool) -> Vec<PlantUmlItem> {
        self.0.as_uml(class, reverse)
    }

    pub fn as_mmd(&self, class: String, reverse: bool) -> Vec<MermaidItem> {
        self.0.as_mmd(class, reverse)
    }

    pub fn as_mem_addr(&self) -> String {
        self.0.display_with_mem_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rev() {
        let path = Path::Start(Scope(1))
            .step('c', Scope(2), 0)
            .step('d', Scope(3), 0);
        println!("{}", path);
        let rev = ReversePath::from(path);
        println!("{}", rev);
    }

    #[test]
    fn test_is_circular() {
        let path = Path::Start(Scope(1))
            .step('c', Scope(2), 0)
            .step('d', Scope(3), 0);
        assert!(!path.is_circular(0));

        let path = Path::Start(Scope(1))
            .step('c', Scope(2), 0)
            .step('d', Scope(3), 0)
            .step('c', Scope(2), 0);
        assert!(path.is_circular(0));

        let path = Path::Start(Scope(1))
            .step('c', Scope(2), 0)
            .step('d', Scope(3), 0)
            .step('c', Scope(2), 1);
        assert!(!path.is_circular(0));
    }
}
