pub struct DisplayVec<'a, T: std::fmt::Display>(pub &'a [T]);

impl<T: std::fmt::Display> std::fmt::Display for DisplayVec<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "[]")
        } else {
            write!(
                f,
                "[{}]",
                self.0
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

pub struct DisplayMap<'a, K: std::fmt::Display, V>(pub &'a std::collections::HashMap<K, V>);

impl<K: std::fmt::Display, T: std::fmt::Display> std::fmt::Display for DisplayMap<'_, K, Vec<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "{{}}")
        } else {
            write!(
                f,
                "{{{}}}",
                self.0
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, DisplayVec(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}
