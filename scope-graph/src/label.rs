pub trait ScopeGraphLabel: PartialEq + std::fmt::Debug {
    fn char(&self) -> char;
    fn str(&self) -> &'static str;
}

impl ScopeGraphLabel for char {
    fn char(&self) -> char {
        *self
    }

    fn str(&self) -> &'static str {
        unimplemented!("char does not have a string representation")
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum LabelOrEnd<Lbl>
{
    Label(Lbl),
    End,
}
