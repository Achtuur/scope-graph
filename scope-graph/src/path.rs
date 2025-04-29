use std::rc::Rc;

use graphing::plantuml::{theme::LineStyle, EdgeDirection, PlantUmlItem};

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
    pub fn step(&self, label: Lbl, scope: Scope) -> Self {
        Self::Step {
            label,
            target: scope,
            from: Rc::new(self.clone()),
        }
    }

    pub fn target(&self) -> Scope {
        match self {
            Self::Start(s) => *s,
            Self::Step { target, .. } => *target,
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
                    from.target().0,
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
    pub fn as_uml(&self, class: String, reverse: bool) -> Vec<PlantUmlItem> {
        match self {
            Self::Start(_) => Vec::new(),
            Self::Step {
                from,
                label,
                target,
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
            } => {
                // what we have: from -L> target
                // what we want: target <L- from

                let mut rp = Path::Start(target);
                rp = rp.step(label, from.target());
                let mut current = from.as_ref();
                while let Path::Step { label, from, .. } = current {
                    rp = rp.step(label.clone(), from.target());
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

    /// Step forward (p -> new p)
    pub fn step(&self, label: Lbl, scope: Scope) -> Self {
        Self(self.0.step(label, scope))
    }

    pub fn as_uml(&self, class: String, reverse: bool) -> Vec<PlantUmlItem> {
        self.0.as_uml(class, reverse)
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
            .step('c', Scope(2))
            .step('d', Scope(3));
        println!("{}", path);
        let rev = ReversePath::from(path);
        println!("{}", rev);
    }
}
