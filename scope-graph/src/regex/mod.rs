use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
enum Regex<Lbl> {
    /// `eps`
    EmptyString,
    /// Empty set, calling it zero to make it immediately distinct from `EmptyString`
    ZeroSet,
    /// `a`
    Character(Lbl),
    /// r . s
    Concat(Rc<Self>, Rc<Self>),
    /// r*
    KleeneStar(Rc<Self>),
    /// r + s
    Or(Rc<Self>, Rc<Self>),
    /// r & s
    And(Rc<Self>, Rc<Self>),
    /// !r
    Neg(Rc<Self>),
}

impl<Lbl> Regex<Lbl>
where Lbl: PartialEq + Clone
{
    pub fn or(r: Self, s: Self) -> Self {
        Self::Or(Rc::new(r), Rc::new(s))
    }

    pub fn and(r: Self, s: Self) -> Self {
        Self::And(Rc::new(r), Rc::new(s))
    }

    pub fn concat(r: Self, s: Self) -> Self {
        Self::Concat(Rc::new(r), Rc::new(s))
    }

    pub fn neg(r: Self) -> Self {
        Self::Neg(Rc::new(r))
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
            Self::And(r, s)
            | Self::Concat(r, s) => match (r.v(), s.v()) {
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
                _ => unreachable!("v should not return anything other than empty set or empty string"),
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
                let lhs = Regex::Concat(Rc::new(r.derivative(dim)), s.clone());
                let rhs = Regex::concat(r.v(), s.derivative(dim));
                Regex::or(lhs, rhs)
            },
            Self::KleeneStar(r) => {
                Regex::Concat(Rc::new(r.derivative(dim)), r.clone())
            },
            Self::Or(r, s) => {
                Regex::or(r.derivative(dim), s.derivative(dim))
            },
            Self::And(r, s) => {
                Regex::and(r.derivative(dim), s.derivative(dim))
            },
            Self::Neg(r) => Regex::neg(r.derivative(dim)),
        }
    }

    pub fn unique_labels(&self) -> Vec<&Lbl> {
        let mut v = match self {
            Self::EmptyString
            | Self::ZeroSet => Vec::new(),
            Self::Character(l) => {
                vec![l]
            },
            Self::Concat(r, s)
            | Self::Or(r, s)
            | Self::And(r, s) => {
                let mut v = Vec::new();
                v.append(&mut r.unique_labels());
                v.append(&mut s.unique_labels());
                v
            },
            Self::KleeneStar(r)
            | Self::Neg(r) => r.unique_labels(),
        };
        v.dedup();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivative() {
        // (a + b) * c
        let r = Regex::or(Regex::Character('a'), Regex::Character('b'));
        let r = Regex::concat(r, Regex::Character('b'));

        // // a * (b + c)
        // let r = Regex::and(Regex::Character('a'), Regex::or(Regex::Character('b'), Regex::Character('c')));

        let lbl = r.unique_labels();
        println!("lbl: {0:?}", lbl);
        let d = r.derivative(&'a');
        // let d = d.derivative(&'a');
        println!("d: {0:?}", d);
        println!("is_nullable: {0:?}", d.is_nullable());
    }
}