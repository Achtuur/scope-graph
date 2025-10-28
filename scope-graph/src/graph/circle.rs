use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Mutex, OnceLock},
};

use hashbrown::HashMap;

use crate::{
    data::ScopeGraphData,
    graph::{ScopeGraph, ScopeMap},
    label::ScopeGraphLabel,
    scope::Scope,
};

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
    where
        Lbl: ScopeGraphLabel,
        Data: ScopeGraphData,
    {
        static QUEUE_ALLOC: OnceLock<Mutex<Vec<CircleMatch>>> = OnceLock::new();
        let mut queue = QUEUE_ALLOC
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .unwrap();
        queue.clear();
        // let mut cur_matches = vec![CircleMatch::from_scope(scope)];
        queue.push(CircleMatch::from_scope(scope));

        while let Some(m) = queue.pop() {
            let Some(outgoing_scopes) = map
                .get(&m.tail())
                .map(|d| d.outgoing().iter().map(|e| e.target()))
            else {
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

    pub fn scopes_in_cycle<Lbl, Data>(
        map: &ScopeMap<Lbl, Data>,
        scope: Scope,
    ) -> hashbrown::HashSet<Scope>
    where
        Lbl: ScopeGraphLabel,
        Data: ScopeGraphData,
    {
        let mut queue = Vec::new();
        let mut found = hashbrown::HashSet::new();
        queue.clear();
        // let mut cur_matches = vec![CircleMatch::from_scope(scope)];
        queue.push(CircleMatch::from_scope(scope));

        while let Some(m) = queue.pop() {
            let Some(outgoing_scopes) = map
                .get(&m.tail())
                .map(|d| d.outgoing().iter().map(|e| e.target()))
            else {
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

    /// Returns (nodes_in_cycles, nodes_not_in_cycles)
    /// Tarjan’s algorithm
    /// I (with shame) asked chatgpt for this
    pub fn find_cycle_nodes<Lbl: ScopeGraphLabel, Data: ScopeGraphData>(
        graph: &ScopeMap<Lbl, Data>,
    ) -> (hashbrown::HashSet<Scope>, hashbrown::HashSet<Scope>) {
        let mut index = 0;
        let mut stack = Vec::new();
        let mut on_stack = hashbrown::HashSet::new();
        let mut indices = HashMap::new();
        let mut lowlink = HashMap::new();
        let mut cycles = hashbrown::HashSet::new();

        fn strongconnect<Lbl2: ScopeGraphLabel, Data2: ScopeGraphData>(
            v: Scope,
            index: &mut i32,
            stack: &mut Vec<Scope>,
            on_stack: &mut hashbrown::HashSet<Scope>,
            indices: &mut HashMap<Scope, i32>,
            lowlink: &mut HashMap<Scope, i32>,
            graph: &ScopeMap<Lbl2, Data2>,
            cycles: &mut hashbrown::HashSet<Scope>,
        ) {
            indices.insert(v, *index);
            lowlink.insert(v, *index);
            *index += 1;
            stack.push(v);
            on_stack.insert(v);

            if let Some(nd) = graph.get(&v) {
                for edge in nd.outgoing() {
                    let w = edge.target();
                    if !indices.contains_key(&w) {
                        strongconnect(w, index, stack, on_stack, indices, lowlink, graph, cycles);
                        let low_v = *lowlink.get(&v).unwrap();
                        let low_w = *lowlink.get(&w).unwrap();
                        lowlink.insert(v, low_v.min(low_w));
                    } else if on_stack.contains(&w) {
                        let low_v = *lowlink.get(&v).unwrap();
                        let idx_w = *indices.get(&w).unwrap();
                        lowlink.insert(v, low_v.min(idx_w));
                    }
                }
            }

            if indices[&v] == lowlink[&v] {
                // Start a new SCC
                let mut scc = Vec::new();
                loop {
                    let w = stack.pop().unwrap();
                    on_stack.remove(&w);
                    scc.push(w);
                    if w == v {
                        break;
                    }
                }

                // If SCC has > 1 node, or a self-loop, it's a cycle
                if scc.len() > 1 {
                    cycles.extend(scc);
                } else if let Some(nd) = graph.get(&scc[0])
                    && nd.outgoing().iter().any(|e| e.target() == scc[0])
                {
                    cycles.insert(scc[0]);
                }
            }
        }

        for &node in graph.keys() {
            if !indices.contains_key(&node) {
                strongconnect(
                    node,
                    &mut index,
                    &mut stack,
                    &mut on_stack,
                    &mut indices,
                    &mut lowlink,
                    graph,
                    &mut cycles,
                );
            }
        }

        let all_nodes: hashbrown::HashSet<Scope> = graph.keys().cloned().collect();
        let non_cycles = &all_nodes - &cycles;

        (cycles, non_cycles)
    }
}

#[derive(Debug)]
pub struct CachedCircleMatcher<'sg, Lbl: ScopeGraphLabel, Data: ScopeGraphData> {
    map: &'sg ScopeMap<Lbl, Data>,
    cache: RefCell<&'sg mut hashbrown::HashMap<Scope, bool>>,
}

impl<'sg, Lbl: ScopeGraphLabel, Data: ScopeGraphData> CachedCircleMatcher<'sg, Lbl, Data> {
    pub fn new(
        map: &'sg ScopeMap<Lbl, Data>,
        cache: &'sg mut hashbrown::HashMap<Scope, bool>,
    ) -> Self {
        Self {
            map,
            cache: RefCell::new(cache),
        }
    }

    pub fn contains(&self, scope: Scope) -> bool {
        return false;
        // let cache = self.cache.borrow();
        if !self.cache.borrow().contains_key(&scope) {
            self.populate_cache();
        }
        self.cache.borrow().get(&scope).cloned().unwrap_or_default()

        // let (in_cycle, not_cycle) = CircleMatcher::scopes_in_cycle(self.map, scope);
        // let mut cache_mut = self.cache.borrow_mut();
        // for scope in in_cycle {
        //     cache_mut.entry(scope).or_insert(true);
        // }
        // for scope in not_cycle {
        //     cache_mut.entry(scope).or_insert(false);
        // }
        // cache_mut.get(&scope).cloned().unwrap_or_default()
        // // contains
    }

    fn populate_cache(&self) {
        let (in_cycle, not_cycle) = CircleMatcher::find_cycle_nodes(self.map);
        let mut cache_mut = self.cache.borrow_mut();
        cache_mut.extend(not_cycle.into_iter().map(|s| (s, false)));
        cache_mut.extend(in_cycle.into_iter().map(|s| (s, true)));

        // let mut in_cycle = hashbrown::HashSet::new();
        // for s in self.map.keys() {
        //     if in_cycle.contains(s) {
        //         continue;
        //     }
        //     let found = CircleMatcher::scopes_in_cycle(self.map, *s);
        //     in_cycle.extend(found);
        // }
        // let not_in_cycle = self.map.keys().filter(|s| !in_cycle.contains(*s));
    }
}

#[cfg(test)]
mod tests {
    use crate::{SgData, SgLabel, graph::CachedScopeGraph};

    use super::*;

    #[test]
    fn test_cycle() {
        let mut graph = CachedScopeGraph::<SgLabel, SgData>::new();
        let s1 = graph.add_scope_default();
        let s2 = graph.add_scope_default();
        let s3 = graph.add_scope_default();
        let s4 = graph.add_scope_default();
        let s5 = graph.add_scope_default();
        graph.add_edge(s1, s2, SgLabel::Parent);
        graph.add_edge(s2, s3, SgLabel::Parent);
        graph.add_edge(s3, s1, SgLabel::Parent);
        graph.add_edge(s4, s1, SgLabel::Parent);
        graph.add_edge(s5, s1, SgLabel::Parent);

        let map = graph.map();

        assert!(CircleMatcher::scope_is_in_cycle(map, s1));
        assert!(CircleMatcher::scope_is_in_cycle(map, s2));
        assert!(CircleMatcher::scope_is_in_cycle(map, s3));
        assert!(!CircleMatcher::scope_is_in_cycle(map, s4));
        assert!(!CircleMatcher::scope_is_in_cycle(map, s5));
    }

    #[test]
    fn test_cached() {
        let mut graph = CachedScopeGraph::<SgLabel, SgData>::new();
        let s1 = graph.add_scope_default();
        let s2 = graph.add_scope_default();
        let s3 = graph.add_scope_default();
        let s4 = graph.add_scope_default();
        let s5 = graph.add_scope_default();
        graph.add_edge(s1, s2, SgLabel::Parent);
        graph.add_edge(s2, s3, SgLabel::Parent);
        graph.add_edge(s3, s1, SgLabel::Parent);
        graph.add_edge(s4, s1, SgLabel::Parent);
        graph.add_edge(s5, s1, SgLabel::Parent);

        let mut map = HashMap::new();
        let matcher = CachedCircleMatcher::new(graph.map(), &mut map);
        assert!(matcher.contains(s1));
        assert!(matcher.contains(s2));
        assert!(matcher.contains(s3));
        assert!(!matcher.contains(s4));
        assert!(!matcher.contains(s5));
        println!("matcher;: {0:?}", matcher);

        let (in_cycle, not_cycle) = CircleMatcher::find_cycle_nodes(graph.map());
        println!("in_cycle: {0:?}", in_cycle);
        println!("not_cycle: {0:?}", not_cycle);
        assert!(in_cycle.contains(&s1));
        assert!(in_cycle.contains(&s2));
        assert!(in_cycle.contains(&s3));
        assert!(not_cycle.contains(&s4));
        assert!(not_cycle.contains(&s5));
    }
}
