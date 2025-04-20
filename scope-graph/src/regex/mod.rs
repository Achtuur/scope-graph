pub mod dfs;

use crate::label::ScopeGraphLabel;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Regex<Lbl>
where Lbl: ScopeGraphLabel
{
    /// `eps`
    EmptyString,
    /// Empty set, calling it zero to make it immediately distinct from `EmptyString`
    ZeroSet,
    /// `a`
    Character(Lbl),
    /// r . s
    Concat(Box<Self>, Box<Self>),
    /// r*
    KleeneStar(Box<Self>),
    /// r + s
    Or(Box<Self>, Box<Self>),
    /// r & s
    And(Box<Self>, Box<Self>),
    /// !r
    Neg(Box<Self>),
}

impl<Lbl> std::fmt::Display for Regex<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyString => write!(f, "ε"),
            Self::ZeroSet => write!(f, "∅"),
            Self::Character(c) => write!(f, "{c}"),
            Self::Concat(r, s) => write!(f, "{r}{s}"), // r dot s
            Self::KleeneStar(r) => write!(f, "{r}*"),
            Self::Or(r, s) => write!(f, "({r}+{s})"),
            Self::And(r, s) => write!(f, "({r}&{s})"),
            Self::Neg(r) => write!(f, "!{r}"),
        }
    }
}

impl<T> From<T> for Regex<T>
where
    T: ScopeGraphLabel + Clone + std::hash::Hash,
{
    fn from(c: T) -> Self {
        Self::Character(c)
    }
}

impl<Lbl> Regex<Lbl>
where
    Lbl: ScopeGraphLabel,
{
    pub fn or(r: impl Into<Self>, s: impl Into<Self>) -> Self {
        Self::Or(Box::new(r.into()), Box::new(s.into()))
    }

    pub fn and(r: impl Into<Self>, s: impl Into<Self>) -> Self {
        Self::And(Box::new(r.into()), Box::new(s.into()))
    }

    pub fn concat(r: impl Into<Self>, s: impl Into<Self>) -> Self {
        Self::Concat(Box::new(r.into()), Box::new(s.into()))
    }

    pub fn kleene(r: impl Into<Self>) -> Self {
        Self::KleeneStar(Box::new(r.into()))
    }

    pub fn neg(r: impl Into<Self>) -> Self {
        Self::Neg(Box::new(r.into()))
    }

    pub fn is_nullable(&self) -> bool {
        self.v() == Regex::EmptyString
    }

    /// Helper function to determine whether a regular expression is final
    pub fn v(&self) -> Regex<Lbl> {
        match self {
            Self::EmptyString => Self::EmptyString,
            Self::ZeroSet => Self::ZeroSet,
            Self::Character(_) => Self::ZeroSet,
            Self::And(r, s) | Self::Concat(r, s) => match (r.v(), s.v()) {
                (Self::EmptyString, Self::EmptyString) => Self::EmptyString,
                _ => Self::ZeroSet,
            },
            Self::KleeneStar(_) => Self::EmptyString,
            Self::Or(r, s) => match (r.v(), s.v()) {
                (Self::EmptyString, _) | (_, Self::EmptyString) => Self::EmptyString,
                _ => Self::ZeroSet,
            },
            Self::Neg(r) => match r.v() {
                Self::EmptyString => Self::ZeroSet,
                Self::ZeroSet => Self::EmptyString,
                _ => unreachable!(
                    "v should not return anything other than empty set or empty string"
                ),
            },
        }
    }

    pub fn derivative(&self, dim: &Lbl) -> Self {
        match self {
            Self::EmptyString => Self::ZeroSet,
            Self::ZeroSet => Self::ZeroSet,
            Self::Character(a) if dim == a => Self::EmptyString,
            Self::Character(_) => Self::ZeroSet, // dim != a
            Self::Concat(r, s) => {
                let lhs = Regex::Concat(Box::new(r.derivative(dim)), s.clone());
                let rhs = Regex::concat(r.v(), s.derivative(dim));
                Regex::or(lhs, rhs)
            }
            Self::KleeneStar(r) => Regex::concat(r.derivative(dim), Regex::KleeneStar(r.clone())),
            Self::Or(r, s) => Regex::or(r.derivative(dim), s.derivative(dim)),
            Self::And(r, s) => Regex::and(r.derivative(dim), s.derivative(dim)),
            Self::Neg(r) => Regex::neg(r.derivative(dim)),
        }
    }

    /// Returns all unique labels in the regex
    pub fn unique_labels(&self) -> Vec<&Lbl> {
        let mut v = match self {
            Self::EmptyString | Self::ZeroSet => Vec::new(),
            Self::Character(l) => {
                vec![l]
            }
            Self::Concat(r, s) | Self::Or(r, s) | Self::And(r, s) => {
                let mut v = Vec::new();
                v.append(&mut r.unique_labels());
                v.append(&mut s.unique_labels());
                v
            }
            Self::KleeneStar(r) | Self::Neg(r) => r.unique_labels(),
        };
        v.dedup();
        v
    }

    /// Returns all leading labels in the regex
    ///
    /// Leading labels are the labels that are not trivially the empty set.
    /// When concatenating two regexes, the leading labels are the labels of the left hand side.
    /// The right hand side is only considered, if the derivative of the left hand side is *not* the empty set
    ///
    /// # Example
    ///
    /// ```rs
    /// // leading labels of `a + bc` are ['a', 'b'].
    /// let r = Regex::or('a', Regex::concat('b', 'c'));
    /// let leading = r.leading_labels();
    /// println!("leading: {0:?}", leading); // ['a', 'b']
    ///
    /// ```
    pub fn leading_labels(&self) -> Vec<&Lbl> {
        let mut v = match self {
            Self::EmptyString | Self::ZeroSet => Vec::new(),
            Self::Character(l) => {
                vec![l]
            }
            // in concat and and, lhs is always considered first
            Self::Concat(r, s) | Self::And(r, s) => {
                let mut v = Vec::new();
                v.append(&mut r.leading_labels());
                // only append right hand side if left is nullable
                // ie P*D should have P and D as leading labels
                if r.is_nullable() {
                    v.append(&mut s.unique_labels());
                }
                v
            }
            Self::Or(r, s) => {
                let mut v = Vec::new();
                v.append(&mut r.leading_labels());
                v.append(&mut s.leading_labels());
                v
            }
            Self::KleeneStar(r) | Self::Neg(r) => r.leading_labels(),
        };
        v.dedup();
        v
    }

    /// Simplify this regex, eg `a + 0` -> `a`, `eps + a -> a`
    pub fn reduce(self) -> Self {
        match self {
            Self::EmptyString => Self::EmptyString,
            Self::ZeroSet => Self::ZeroSet,
            Self::Character(_) => self,
            Self::And(r, s) | Self::Concat(r, s) => match (r.reduce(), s.reduce()) {
                (Self::ZeroSet, _) | (_, Self::ZeroSet) => Self::ZeroSet,
                (Self::EmptyString, s) => s,
                (r, Self::EmptyString) => r,
                (r, s) => Self::concat(r, s),
            },
            Self::KleeneStar(r) => match r.reduce() {
                Self::ZeroSet => Self::ZeroSet,
                Self::EmptyString => Self::EmptyString,
                r => Self::KleeneStar(Box::new(r)),
            },
            Self::Or(r, s) => match (r.reduce(), s.reduce()) {
                (Self::EmptyString, Self::ZeroSet) | (Self::ZeroSet, Self::EmptyString) => {
                    Self::EmptyString
                }
                (Self::ZeroSet | Self::EmptyString, s) => s,
                (r, Self::ZeroSet | Self::EmptyString) => r,
                (r, s) => Self::or(r, s),
            },
            Self::Neg(r) => Self::neg(r.reduce()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivative() {
        let r = Regex::kleene('b');
        let d = r.derivative(&'b');
        println!("d: {0:?}", d);
    }

    #[test]
    fn test_leading_label() {
        let r = Regex::or('a', Regex::concat('b', 'c'));
        // let r = Regex::or(
        //     Regex::concat('a', 'b'),
        //     Regex::concat('a', 'c'),
        // );
        let leading = r.leading_labels();
        println!("leading: {0:?}", leading);
    }
}
