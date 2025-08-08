mod segment;

use std::{cell::{OnceCell, RefCell}, collections::HashSet, rc::Rc, sync::{LazyLock, Mutex, OnceLock}};

use deepsize::DeepSizeOf;
use graphing::{
    mermaid::{item::MermaidItem, theme::EdgeType},
    plantuml::{EdgeDirection, PlantUmlItem},
};

use crate::{label::ScopeGraphLabel, path::segment::PathSegment, scope::Scope, util::ContainsContainer, DO_CIRCLE_CHECK};

/// Path enum "starts" at the target scope, ie its in reverse order
///
/// This holds a path using a pointer to the head path segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(DeepSizeOf)]
pub enum Path<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    Start(Scope),
    Step {
        automaton_idx: usize,
        label: Lbl,
        target: Scope,
        len: usize,
        from: Rc<Self>,
    },
}

impl<Lbl: ScopeGraphLabel> Path<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub fn start(start: impl Into<Scope>) -> Self {
        Self::Start(start.into())
    }

    /// Step forward (p -> new p)
    #[inline]
    pub fn step(&self, label: Lbl, scope: impl Into<Scope>, automaton_idx: usize) -> Self {
        Self::Step {
            label,
            target: scope.into(),
            from: Rc::new(self.clone()),
            automaton_idx,
            len: self.len() + 1,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Start(_) => 1,
            Self::Step { len, .. } => *len,
        }
    }

    #[inline]
    pub fn target(&self) -> Scope {
        *self.target_ref()
    }

    #[inline]
    fn target_ref(&self) -> &Scope {
        match self {
            Self::Start(s) => s,
            Self::Step { target, .. } => target,
        }
    }

    pub fn automaton_idx(&self) -> usize {
        match self {
            Self::Start(_) => 0,
            Self::Step { automaton_idx, .. } => *automaton_idx,
        }
    }

    pub fn start_scope(&self) -> Scope {
        match self {
            Self::Start(s) => *s,
            Self::Step { from, .. } => from.start_scope(),
        }
    }

    pub fn without_head(&self) -> Option<&Self> {
        match self {
            Self::Start(_) => None,
            Self::Step { from, .. } => Some(from),
        }
    }

    pub fn without_head_unless_start(&self) -> &Self {
        match self {
            Self::Start(_) => self,
            Self::Step { from, .. } => from,
        }
    }


    /// Returns true if `other` is partially contained within this path.
    pub fn partially_contains(&self, other: &Self) -> bool {
        if self.len() < other.len() {
            return false;
        }

        let mut visited = ContainsContainer::<_, 16>::with_capacity(self.len());

        for s in self.iter() {
            visited.insert(s.target_ref());
        }

        for o in other.iter() {
            if visited.contains(o.target_ref()) {
                return true;
            }
        }

        false
    }

    /// Returns true if `other` is contained within this path.
    ///
    /// This means that `other.len() < self.len()`
    ///
    /// This function is currently very expensive to run
    fn contains<'a>(&self, other: &Self) -> bool
    where Lbl: 'a
    {
        if other.len() > self.len() {
            return false;
        }

        for i in 0..=(self.len() - other.len()) {
            let self_seg = PathSegment::from_path_with_offset(self, i, other.len());
            let other_seg = PathSegment::from_path(other);

            let is_eq = self_seg
            .zip(other_seg)
            .all(|(s, o)| {
                s.equals(&o)
            });

            if is_eq {
                return true;
            }
        }
        false
    }

    pub fn iter<'a>(&'a self) -> PathIterator<'a, Lbl> {
        PathIterator { current: Some(self) }
    }

    pub fn parent(&self) -> Option<&Self> {
        match self {
            Self::Start(_) => None,
            Self::Step { from, .. } => Some(from),
        }
    }

    pub fn is_circular2(&self) -> bool {
        let mut slow = self;
        let mut fast = self;
        loop {
            slow = match slow.parent() {
                Some(s) => s,
                None => return false,
            };

            fast = match fast.parent() {
                Some(f) => match f.parent() {
                    Some(ff) => ff,
                    None => return false,
                },
                None => return false,
            };
            if slow.target() == fast.target() && slow.automaton_idx() == fast.automaton_idx() {
                return true;
            }
        }
    }

    pub fn is_circular(&self) -> bool {
        // todo: pass hashset as argument maybe?
        static SET: OnceLock<Mutex<hashbrown::HashSet<(Scope, usize)>>> = OnceLock::new();
        let mut current = self;
        let mut set = SET.get_or_init(|| Mutex::new(hashbrown::HashSet::new())).lock().unwrap();
        set.clear();
        let mut prev_index = 0;
        loop {
            match current {
                Self::Start(s) => return set.contains(&(*s, 0)),
                Self::Step {
                    target,
                    from,
                    automaton_idx,
                    ..
                } => {
                    if set.contains(&(*target, prev_index)) {
                        return true;
                    }
                    unsafe { set.insert_unique_unchecked((*target, prev_index)); }
                    // set.insert((*target, prev_index));
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
            Self::Step { from, target, .. } => {
                let (from_scope, to_scope) = match reverse {
                    false => (from.target(), *target),
                    true => (*target, from.target()),
                };

                let item = PlantUmlItem::edge(
                    from_scope.uml_id(),
                    to_scope.uml_id(),
                    "",
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
            Self::Start(s) => format!("{s}"),
            Self::Step {
                from,
                label,
                target,
                automaton_idx,
                ..
            } => {
                format!("{} -{}{}-> {}", from.display(), label.char(), automaton_idx, target)
            }
        }
    }

    pub fn display_with_mem_addr(&self) -> String {
        match self {
            Self::Start(s) => format!("{s}"),
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

#[derive(Clone)]
pub struct PathIterator<'a, Lbl: ScopeGraphLabel> {
    current: Option<&'a Path<Lbl>>,
}

impl<'a, Lbl: ScopeGraphLabel> Iterator for PathIterator<'a, Lbl> {
    type Item = &'a Path<Lbl>;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.current;
        self.current = match self.current {
            Some(Path::Start(_)) | None => None,
            Some(Path::Step {from, ..}) => Some(from)
        };
        ret
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
#[derive(DeepSizeOf)]
#[repr(transparent)]
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
                ..
            } => {
                // what we have: from -L> target
                // what we want: target <L- from

                let mut rp = Path::Start(target);
                rp = rp.step(label, from.target(), automaton_idx);
                let mut current = from.as_ref();
                while let Path::Step { label, from, automaton_idx, .. } = current {
                    rp = rp.step(label.clone(), from.target(), *automaton_idx);
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
    #[inline(always)]
    pub fn start(scope: Scope) -> Self {
        Self(Path::start(scope))
    }

    /// Gets the target of the current path.
    ///
    /// This is equal to Path::start_scope()
    #[inline(always)]
    pub fn target(&self) -> Scope {
        self.0.start_scope()
    }

    /// Gets the start scope of the current path.
    ///
    /// This is equal to Path::target()
    #[inline(always)]
    pub fn start_scope(&self) -> Scope {
        self.0.target()
    }

    #[inline(always)]
    pub fn is_circular(&self) -> bool {
        self.0.is_circular()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    pub fn partially_contains(&self, other: &Self) -> bool {
        self.0.partially_contains(&other.0)
    }

    /// Step forward (p -> new p)
    #[inline(always)]
    pub fn step(&self, label: Lbl, scope: Scope, automaton_idx: usize) -> Self {
        Self(self.0.step(label, scope, automaton_idx))
    }

    #[inline(always)]
    pub fn as_uml(&self, class: String, reverse: bool) -> Vec<PlantUmlItem> {
        self.0.as_uml(class, reverse)
    }

    #[inline(always)]
    pub fn as_mmd(&self, class: String, reverse: bool) -> Vec<MermaidItem> {
        self.0.as_mmd(class, reverse)
    }

    #[inline(always)]
    pub fn as_mem_addr(&self) -> String {
        self.0.display_with_mem_addr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rev() {
        let path: Path<char> = Path::Start(Scope(1))
            .step('c', Scope(2), 0)
            .step('d', Scope(3), 0);
        println!("{}", path);
        let rev = ReversePath::from(path);
        println!("{}", rev);
    }

    #[test]
    fn test_is_circular() {
        let path: Path<char> = Path::Start(Scope(1))
            .step('a', Scope(2), 0)
            .step('b', Scope(3), 0);
        assert!(!path.is_circular());

        let path: Path<char> = Path::Start(Scope(1))
            .step('c', Scope(2), 0)
            .step('d', Scope(3), 0)
            .step('c', Scope(2), 0);
        assert!(path.is_circular());

        // todo: fix automaton index
        // let path: Path<char> = Path::Start(Scope(1))
        //     .step('c', Scope(2), 0)
        //     .step('d', Scope(3), 1)
        //     .step('c', Scope(2), 1);
        // assert!(!path.is_circular());

        let path = Path::start(4)
            .step('d', 0, 0)
            .step('p', 3, 0)
            .step('p', 2, 0)
            .step('p', 1, 0)
            .step('p', 0, 0);
        println!("path: {0:?}", path);
        println!("path: {}", path);
        assert!(path.is_circular());

    }

    #[test]
    fn test_equality() {
        let p1 = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        let p2 = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        assert_eq!(p1, p2);
        let p3 = Path::start(3).step('a', 2, 0).step('b', 3, 0);
        assert_ne!(p1, p3);
        let p4 = Path::start(1).step('a', 2, 0).step('c', 3, 0);
        assert_ne!(p1, p4);
    }

    #[test]
    fn test_deepsize() {
        let p1 = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        let p2 = p1.clone().step('a', 3, 0);
        let p3 = p2.clone().step('a', 3, 0);

        let p1z = p1.deep_size_of() as f32;
        let p2z = p2.deep_size_of() as f32;
        let p3z = p3.deep_size_of() as f32;
        println!("p2z/p1z: {0:?}", p2z/p1z);
        println!("p2z/p1z: {0:?}", p3z/p2z);

        // let empty_v = Vec::<usize>::new();
        // println!("empty_v.deep_size_of():; {0:?}", empty_v.deep_size_of());

        // // this should be same as vec + p2, since they point to the same memory
        let v = vec![p1.clone(), p2.clone(), p3.clone()];
        let v2 = vec![p1, p2, p3];
        println!("v.deep_size_of(): {0:?}", (v, v2).deep_size_of());

    }

    #[test]
    fn test_contains() {
        let p1: Path<char> = Path::start(1);
        let p2: Path<char> = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        assert!(p2.contains(&p1));

        let p1: Path<char> = Path::start(1).step('a', 2, 0);
        let p2: Path<char> = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        assert!(p2.contains(&p1));


        let p1 = Path::start(1).step('a', 2, 0).step('b', 2, 1);
        let p2 = Path::start(2).step('a', 2, 0).step('b', 2, 1);
        assert!(!p1.contains(&p2));

        let p1 = Path::start(1).step('a', 2, 0).step('b', 3, 1);
        let p2 = Path::start(2).step('a', 2, 0).step('b', 4, 1);
        assert!(!p1.contains(&p2));
    }

    #[test]
    fn test_partially_contains() {
        let p1: Path<char> = Path::start(1);
        let p2: Path<char> = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        assert!(p2.partially_contains(&p1));

        // start corresponds to end of path
        let p1: Path<char> = Path::start(2);
        let p2: Path<char> = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        assert!(p2.partially_contains(&p1));

        // end is different
        let p1: Path<char> = Path::start(1).step('a', 2, 0).step('c', 3, 0);
        let p2: Path<char> = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        assert!(p2.partially_contains(&p1));

        let p1: Path<char> = Path::start(0).step('d', 1, 0).step('a', 2, 0).step('c', 3, 0);
        let p2: Path<char> = Path::start(1).step('a', 2, 0).step('b', 3, 0);
        assert!(p2.partially_contains(&p1));

        // different
        let p1: Path<char> = Path::start(0).step('d', 1, 0).step('a', 2, 0).step('c', 3, 0);
        let p2: Path<char> = Path::start(4).step('a', 5, 0).step('b', 6, 0);
        assert!(!p2.partially_contains(&p1));
    }
}
