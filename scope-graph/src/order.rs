use std::{
    collections::{BTreeMap, HashSet, btree_map::Entry},
    fmt::Write,
    hash::Hash,
};

use deepsize::DeepSizeOf;

use crate::label::{LabelOrEnd, ScopeGraphLabel};

pub struct LabelOrderBuilder<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    /// graph containing labels and orderings.
    /// If an edge exists from a label to another label, then the source node has a higher priority
    /// ie if `graph.get('a') = ['b']`, then a < b
    graph: BTreeMap<Lbl, Vec<Lbl>>,
    all_labels: HashSet<Lbl>,
}

// use fullwidth_lt since mmd doesnt render '<' properly
const FULLWIDTH_LT: char = '＜';

impl<Lbl> Default for LabelOrderBuilder<Lbl>
where
    Lbl: ScopeGraphLabel + Clone + Hash + Eq + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Lbl> LabelOrderBuilder<Lbl>
where
    Lbl: ScopeGraphLabel + Clone + Hash + Eq + Ord,
{
    pub fn new() -> Self {
        Self {
            graph: BTreeMap::new(),
            all_labels: HashSet::new(),
        }
    }

    pub fn push(mut self, lhs: Lbl, rhs: Lbl) -> Self {
        self.all_labels.insert(lhs.clone());
        self.all_labels.insert(rhs.clone());
        match self.graph.entry(lhs) {
            // add edge
            Entry::Occupied(mut occ) => {
                occ.get_mut().push(rhs);
            }

            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(vec![rhs]);
            }
        }
        self
    }

    pub fn build(self) -> LabelOrder<Lbl> {
        let mut orders = Vec::new();

        for lbl in &self.all_labels {
            let mut less_thans = Vec::new();
            for lbl2 in &self.all_labels {
                if lbl == lbl2 {
                    continue;
                }
                if self.cmp(lbl, lbl2).is_lt() {
                    less_thans.push(lbl2.clone());
                }
            }
            // order should be stable if the same order is built multiple times
            // if not, then multiple cache entries are created
            less_thans.sort();
            orders.push((lbl.clone(), less_thans));
        }
        orders.sort();
        LabelOrder { orders }
    }

    /// Returns the ordering of two labels w.r.t. `label1`
    fn cmp(&self, label1: &Lbl, label2: &Lbl) -> std::cmp::Ordering {
        if label1 == label2 {
            return std::cmp::Ordering::Equal;
        }

        let res = match (
            self.traverse_graph(label1, label2),
            self.traverse_graph(label2, label1),
        ) {
            (Some(l1), Some(l2)) => {
                eprintln!(
                    "Circular label order: {0:?} < {1:?} while {0:?} > {1:?}",
                    l1, l2
                );
                panic!("Circular ordering")
            }
            (Some(_), None) => {
                // println!("{:?} < {:?}", label1, label2);
                std::cmp::Ordering::Less
            }
            (None, Some(_)) => {
                // println!("{:?} > {:?}", label1, label2);
                std::cmp::Ordering::Greater
            }
            (None, None) => {
                // println!("{:?} = {:?}", label1, label2);
                std::cmp::Ordering::Equal
            }
        };
        res
    }

    /// Less, so HIGHER priority
    pub fn is_less(&self, label1: &LabelOrEnd<Lbl>, label2: &LabelOrEnd<Lbl>) -> bool {
        match (label1, label2) {
            (LabelOrEnd::End, LabelOrEnd::End) => false,
            (LabelOrEnd::End, LabelOrEnd::Label(_)) => true,
            (LabelOrEnd::Label(_), LabelOrEnd::End) => false,
            (LabelOrEnd::Label((l1, _)), LabelOrEnd::Label((l2, _))) => self.cmp(l1, l2).is_lt(),
        }
    }

    fn traverse_graph<'a>(&'a self, lbl: &'a Lbl, end: &'a Lbl) -> Option<&'a Lbl> {
        if lbl == end {
            return Some(end);
        }

        // traverse all edges (breadth first search) to find match
        let edges = self.graph.get(lbl)?;
        for e in edges {
            if let Some(lbl) = self.traverse_graph(e, end) {
                return Some(lbl);
            }
        }
        None
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
#[derive(DeepSizeOf)]
pub struct LabelOrder<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    /// Label orderings
    /// First vec contains all labels
    /// Second vec contains all labels that are less than the first label
    orders: Vec<(Lbl, Vec<Lbl>)>,
}

impl<Lbl> LabelOrder<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    /// Less, so HIGHER priority
    pub fn is_less(&self, label1: &LabelOrEnd<Lbl>, label2: &LabelOrEnd<Lbl>) -> bool {
        match (label1, label2) {
            (LabelOrEnd::End, LabelOrEnd::End) => false,
            (LabelOrEnd::End, LabelOrEnd::Label(_)) => true,
            (LabelOrEnd::Label(_), LabelOrEnd::End) => false,
            (LabelOrEnd::Label((l1, _)), LabelOrEnd::Label((l2, _))) => {
                self.is_less_internal(l1, l2)
            }
        }
    }

    // returns true if lbl 1 is less than label2 (so higher priority)
    fn is_less_internal(&self, lbl1: &Lbl, lbl2: &Lbl) -> bool {
        let Some((_, less_thans)) = self.orders.iter().find(|(l, _)| l == lbl1) else {
            return false;
        };
        less_thans.iter().any(|l| l == lbl2)
    }
}

impl<Lbl> std::fmt::Display for LabelOrder<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self
            .orders
            .iter()
            .flat_map(|(lbl, less_thans)| {
                less_thans
                    .iter()
                    .map(|lt| format!("{} {} {}", lbl.char(), FULLWIDTH_LT, lt.char()))
                // let less_than_str = less_thans
                // .iter()
                // .fold(String::new(), |mut s, lt| {
                //     write!(&mut s, "{} {} {}, ", lbl.char(), FULLWIDTH_LT, lt.char())
                //         .expect("Failed to write string");
                //     s
                // });
                // less_than_str.trim_end_matches(", ").to_string()
            })
            .collect::<Vec<_>>()
            .join(", ");

        write!(f, "{}", s.trim_end_matches(" "))
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn test_inference() {
        let order = LabelOrderBuilder::new()
            .push('a', 'b')
            .push('a', 'c')
            .push('b', 'c')
            .push('a', 'd')
            .build();

        println!("order: {0:?}", order);

        assert!(order.is_less_internal(&'a', &'b'));
        assert!(!order.is_less_internal(&'b', &'a'));
        assert!(order.is_less_internal(&'a', &'c'));
        assert!(!order.is_less_internal(&'c', &'a'));
        assert!(order.is_less_internal(&'a', &'d'));
        assert!(!order.is_less_internal(&'d', &'a'));
        assert!(!order.is_less_internal(&'b', &'d'));
        assert!(!order.is_less_internal(&'c', &'d'));
        assert!(!order.is_less_internal(&'d', &'c'));
    }

    #[test]
    #[should_panic]
    fn test_circular_order() {
        let order = LabelOrderBuilder::new().push('a', 'b');
        assert_eq!(order.cmp(&'a', &'b'), Ordering::Less);

        let order = order.push('b', 'a');
        // should panic
        order.cmp(&'a', &'b');
    }
}
