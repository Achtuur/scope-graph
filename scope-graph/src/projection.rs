use crate::data::ScopeGraphData;

pub trait ScopeGraphDataProjection<D: ScopeGraphData>: std::hash::Hash + Eq {
    type Output: std::hash::Hash + Eq;

    fn project(&self, data: &D) -> Self::Output
    where
        D: ScopeGraphData;
}

impl<D> ScopeGraphDataProjection<D> for ()
where
    D: ScopeGraphData,
{
    type Output = ();

    fn project(&self, _: &D) -> Self::Output
    where
        D: ScopeGraphData,
    {
    }
}

impl<D, F, O> ScopeGraphDataProjection<D> for F
where
    D: ScopeGraphData,
    O: std::hash::Hash + Eq,
    F: for<'d> Fn(&'d D) -> O + Eq + std::hash::Hash,
{
    type Output = O;

    fn project(&self, data: &D) -> Self::Output
    where
        D: ScopeGraphData,
    {
        (self)(data)
    }
}
