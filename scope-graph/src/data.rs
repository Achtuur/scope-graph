pub trait ScopeGraphData {

    /// Returns true if the variant has data.
    ///
    /// If have a data variant that contains no data, return false.
    fn variant_has_data(&self) -> bool;
    /// String to use when rendering the data
    fn render_string(&self) -> String;
}