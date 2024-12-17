use crate::label::ScopeGraphLabel;

pub(crate) struct LabelOrder<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    /// Orders of labels, el.0 < el.1
    order: Vec<(Lbl, Lbl)>,
}

impl<Lbl> LabelOrder<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub fn new() -> Self {
        Self { order: Vec::new() }
    }

    pub fn push(mut self, lhs: Lbl, rhs: Lbl) -> Self {
        self.order.push((lhs, rhs));
        self
    }

    pub fn contains(&self, label: &Lbl) -> bool {
        self.order
            .iter()
            .any(|(lhs, rhs)| lhs == label || rhs == label)
    }

    pub fn cmp(&self, label1: &Lbl, label2: &Lbl) -> std::cmp::Ordering {
        for (lhs, rhs) in &self.order {
            match (lhs, rhs) {
                (l, r) if l == label1 && r == label2 => return std::cmp::Ordering::Less,
                (l, r) if l == label2 && r == label1 => return std::cmp::Ordering::Greater,
                _ => (),
            }
        }
        std::cmp::Ordering::Equal
    }
}
