//! Optional portable stroke geometry policies; absent controls preserve legacy primitives.
/// Endpoint shape of an open solid or dashed stroke run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LineEnd {
    /// End at the authored endpoint without extension.
    #[default]
    Butt,
    /// Extend by a semicircle of radius half the stroke width.
    Round,
    /// Extend by half the stroke width along the endpoint tangent.
    Square,
}
/// Corner shape of a stroked path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LineJoin {
    /// Intersect exterior edges, with the reference miter limit of ten; otherwise bevel.
    #[default]
    Miter,
    /// Join exterior edges with a circular arc.
    Round,
    /// Join exterior edges by their straight chord.
    Bevel,
}
