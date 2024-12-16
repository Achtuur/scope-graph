use std::rc::Rc;

use crate::{label::ScopeGraphLabel, path::Path};


#[derive(Debug, PartialEq)]
pub enum LabelRegex<Lbl: ScopeGraphLabel> {
    /// Match a single label
    Single(Lbl),
    /// Match zero or more
    ZeroOrMore(Lbl),
    /// Match one or more
    OneOrMore(Lbl),
    Or(Rc<Self>, Rc<Self>),
}


impl<Lbl: ScopeGraphLabel + Clone> LabelRegex<Lbl> {
    pub fn or(reg1: Self, reg2: Self) -> Self {
        Self::Or(Rc::new(reg1), Rc::new(reg2))
    }

    fn full_match(&self, path: &mut Vec<&Lbl>) -> bool {
        match self {
            LabelRegex::Single(l) => {
                if !path.is_empty() && path[0] == l {
                    path.remove(0);
                    return true;
                }
                false
            },
            LabelRegex::ZeroOrMore(l) => {
                while !path.is_empty() && path[0] == l {
                    path.remove(0);
                }
                true
            },
            LabelRegex::OneOrMore(l) => {
                Self::Single(l.clone()).full_match(path) && Self::ZeroOrMore(l.clone()).full_match(path)
            },
            LabelRegex::Or(l1, l2) => {
                l1.full_match(path) || l2.full_match(path)
            },
        }
    }
}

pub struct LabelRegexMatcher<Lbl: ScopeGraphLabel> {
    regex: Vec<LabelRegex<Lbl>>,
}

impl<Lbl: ScopeGraphLabel + Clone> LabelRegexMatcher<Lbl> {
    pub fn new(regex: Vec<LabelRegex<Lbl>>) -> Self {
        Self {
            regex,
        }
    }

    pub fn full_match(&self, path: &Path<Lbl>) -> bool {
        let mut path_vec = path
        .as_lbl_vec()
        .into_iter()
        .rev()
        .collect();
        for reg in self.regex.iter() {
            if !reg.full_match(&mut path_vec) {
                return false;
            }
        }
        path_vec.is_empty()
    }

    pub fn partial_match(&self, path: &Path<Lbl>) -> bool {
        let mut path_vec = path
        .as_lbl_vec()
        .into_iter()
        .rev()
        .collect();
        for reg in self.regex.iter() {
            if !reg.full_match(&mut path_vec) {
                return false;
            }

            // exit early if path is empty, this means that every regex up till now matched
            if path_vec.is_empty() {
                return true;
            }
        }
        path_vec.is_empty()
    }
}