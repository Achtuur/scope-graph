use std::{collections::{hash_map::Entry, HashMap, HashSet}, hash::Hash};

use crate::label::ScopeGraphLabel;

pub(crate) struct LabelOrder<Lbl>
where
    Lbl: ScopeGraphLabel + Hash + Eq,
{
    /// graph containing labels and orderings.
    /// If an edge exists from a label to another label, then the source node has a higher priority
    /// ie if `graph.get('a') = ['b']`, then a < b
    graph: HashMap<Lbl, Vec<Lbl>>,
}

// use fullwidth_lt since mmd doesnt render '<' properly
const FULLWIDTH_LT: char = '＜';

impl<Lbl> std::fmt::Display for LabelOrder<Lbl>
where
    Lbl: ScopeGraphLabel + std::fmt::Display + Hash + Eq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = "todo: display label order";

        write!(f, "{}", s.trim_end_matches(" "))
    }
}

impl<Lbl> LabelOrder<Lbl>
where
    Lbl: ScopeGraphLabel + Clone + Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }

    pub fn push(mut self, lhs: Lbl, rhs: Lbl) -> Self {
        match self.graph.entry(lhs) {
            // add edge
            Entry::Occupied(mut occ) => {
                occ.get_mut().push(rhs);
            },

            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(vec![rhs]);
            }
        }
        self
    }

    pub fn contains(&self, label: &Lbl) -> bool {
        self.graph.contains_key(label)
    }

    // pub fn cmp(&self, label1: &Lbl, label2: &Lbl) -> std::cmp::Ordering {
    //     for (lhs, rhs) in &self.order {
    //         match (lhs, rhs) {
    //             (l, r) if l == label1 && r == label2 => return std::cmp::Ordering::Less,
    //             (l, r) if l == label2 && r == label1 => return std::cmp::Ordering::Greater,
    //             _ => (),
    //         }
    //     }
    //     std::cmp::Ordering::Equal
    // }

    /// Returns the ordering of two labels w.r.t. `label1`
    pub fn cmp(&self, label1: &Lbl, label2: &Lbl) -> std::cmp::Ordering {
        if label1 == label2 {
            return std::cmp::Ordering::Equal
        }

        let res = match (self.traverse_graph(label1, label2), self.traverse_graph(label2, label1)) {
            (Some(l1), Some(l2)) => {
                eprintln!("Circular label order: {0:?} < {1:?} while {0:?} > {1:?}", l1, l2);
                panic!("Circular ordering")
            },
            (Some(_), None) => {
                // println!("{:?} < {:?}", label1, label2);
                std::cmp::Ordering::Less
            },
            (None, Some(_)) => {
                // println!("{:?} > {:?}", label1, label2);
                std::cmp::Ordering::Greater
            },
            (None, None) => {
                // println!("{:?} = {:?}", label1, label2);
                std::cmp::Ordering::Equal
            },
        };
        res
    }

    fn traverse_graph<'a>(&'a self, lbl: &'a Lbl, end: &'a Lbl) -> Option<&'a Lbl> {
        if lbl == end {
            return Some(end)
        }

        // traverse all edges (breadth first search) to find match
        let edges = self.graph.get(lbl)?;
        for e in edges {
            if let Some(lbl) = self.traverse_graph(e, end) {
                return Some(lbl)
            }
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn test_inference() {
        let order = LabelOrder::new()
        .push('a', 'b')
        .push('a', 'c')
        .push('b', 'c')
        .push('a', 'd');

        assert_eq!(order.cmp(&'a', &'b'), Ordering::Less);
        assert_eq!(order.cmp(&'b', &'a'), Ordering::Greater);
        assert_eq!(order.cmp(&'a', &'c'), Ordering::Less);
        assert_eq!(order.cmp(&'c', &'a'), Ordering::Greater);
        assert_eq!(order.cmp(&'a', &'d'), Ordering::Less);
        assert_eq!(order.cmp(&'d', &'a'), Ordering::Greater);
        assert_eq!(order.cmp(&'b', &'d'), Ordering::Equal);
        assert_eq!(order.cmp(&'c', &'d'), Ordering::Equal);
        assert_eq!(order.cmp(&'d', &'c'), Ordering::Equal);
    }

    #[test]
    #[should_panic]
    fn test_circular_order() {
        let order = LabelOrder::new().push('a', 'b');
        assert_eq!(order.cmp(&'a', &'b'), Ordering::Less);

        let order = order.push('b', 'a');
        // should panic
        order.cmp(&'a', &'b');
    }
}