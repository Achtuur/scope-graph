use std::{rc::Rc, sync::{Mutex, OnceLock}};

use hashbrown::HashMap;

use crate::{data::ScopeGraphData, graph::{ScopeGraph, ScopeMap}, label::ScopeGraphLabel, scope::Scope};


#[derive(Clone, Debug)]
pub(crate) struct ChainScope {
    pub(crate) s: Scope,
    pub(crate) parent: Option<Rc<ChainScope>>,
}

impl ChainScope {
    fn scopes(&self) -> impl Iterator<Item = &Scope> {
        ChainScopeIter {
            current: Some(self),
        }
    }
}

pub(crate) struct ChainScopeIter<'a> {
    pub(crate) current: Option<&'a ChainScope>,
}

impl<'a> Iterator for ChainScopeIter<'a> {
    type Item = &'a Scope;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.current?;
        self.current = cur.parent.as_deref();
        Some(&cur.s)
    }
}


#[derive(Clone, Debug)]
pub struct CircleMatch {
    first: Scope,
    size: usize,
    nodes: ChainScope,
}

unsafe impl Send for CircleMatch {}

impl CircleMatch {
    pub fn from_scope(scope: Scope) -> Self {
        CircleMatch {
            first: scope,
            size: 1,
            nodes: ChainScope {
                s: scope,
                parent: None,
            },
        }
    }

    pub fn tail(&self) -> Scope {
        self.nodes.s
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn scopes(&self) -> impl Iterator<Item = &Scope> {
        std::iter::once(&self.first).chain(self.nodes.scopes())
    }

    pub fn contains(&self, c: &Scope) -> bool {
        self.nodes.scopes().any(|s| s == c)
    }

    pub fn step(self, scope: Scope) -> Self {
        Self {
            first: self.first,
            size: self.size + 1,
            nodes: ChainScope {
                s: scope,
                parent: Some(Rc::new(self.nodes)),
            },
        }
    }

    pub fn is_circular(&self) -> bool {
        self.first == self.tail()
    }
}

pub struct CircleMatcher;

impl CircleMatcher {
    pub fn scope_is_in_cycle<Lbl, Data>(map: &ScopeMap<Lbl, Data>, scope: Scope) -> bool
    where Lbl: ScopeGraphLabel, Data: ScopeGraphData
    {
        static QUEUE_ALLOC: OnceLock<Mutex<Vec<CircleMatch>>> = OnceLock::new();
        let mut queue = QUEUE_ALLOC.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
        queue.clear();
        // let mut cur_matches = vec![CircleMatch::from_scope(scope)];
        queue.push(CircleMatch::from_scope(scope));

        while let Some(m) = queue.pop() {
            let Some(outgoing_scopes) = map.get(&m.tail()).map(|d| d.outgoing().iter().map(|e| e.target())) else {
                continue;
            };
            for s in outgoing_scopes {
                let step = m.clone().step(s);
                if step.is_circular() {
                    return true;
                } else if !m.contains(&s) {
                    queue.push(step);
                }
            }
        }
        false
    }

    pub fn scopes_in_cycle<Lbl, Data>(map: &ScopeMap<Lbl, Data>, scope: Scope) -> hashbrown::HashSet<Scope>
    where Lbl: ScopeGraphLabel, Data: ScopeGraphData
    {
        let mut queue = Vec::new();
        let mut found = hashbrown::HashSet::new();
        queue.clear();
        // let mut cur_matches = vec![CircleMatch::from_scope(scope)];
        queue.push(CircleMatch::from_scope(scope));

        while let Some(m) = queue.pop() {
            let Some(outgoing_scopes) = map.get(&m.tail()).map(|d| d.outgoing().iter().map(|e| e.target())) else {
                continue;
            };
            for s in outgoing_scopes {
                let step = m.clone().step(s);
                if step.is_circular() {
                    for s in step.scopes() {
                        found.insert(*s);
                    }
                } else if !m.contains(&s) {
                    queue.push(step);
                }
            }
        }

        found
    }
}

#[cfg(test)]
mod tests {
    use crate::{graph::CachedScopeGraph, SgData, SgLabel};

    use super::*;

    #[test]
    fn test_cycle() {
        let mut graph = CachedScopeGraph::<SgLabel, SgData>::new();
        let s1 = graph.add_scope_default();
        let s2 = graph.add_scope_default();
        let s3 = graph.add_scope_default();
        let s4 = graph.add_scope_default();
        graph.add_edge(s1, s2, SgLabel::Parent);
        graph.add_edge(s2, s3, SgLabel::Parent);
        graph.add_edge(s3, s1, SgLabel::Parent);
        graph.add_edge(s4, s1, SgLabel::Parent);

        let map = graph.map();

        assert!(CircleMatcher::scope_is_in_cycle(map, s1));
        assert!(CircleMatcher::scope_is_in_cycle(map, s2));
        assert!(CircleMatcher::scope_is_in_cycle(map, s3));
        assert!(!CircleMatcher::scope_is_in_cycle(map, s4));
    }
}