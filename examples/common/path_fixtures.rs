//! FIX-P06 reusable primary path figure for native and headless consumers.
use chart_core::{
    composition::Anchor,
    path::PathRequest,
    prelude::*,
    scene::{Color, Stroke},
};
#[derive(serde::Deserialize)]
pub struct RenderCase {
    pub id: String,
    pub anchor: Anchor,
    pub operations: Vec<PathOp>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
    pub overflow: bool,
    #[serde(default)]
    pub transform: Option<[f64; 6]>,
}
#[derive(serde::Deserialize)]
pub struct RenderCases {
    pub version: u32,
    pub width: f64,
    pub height: f64,
    pub cases: Vec<RenderCase>,
}
pub fn cases() -> RenderCases {
    serde_json::from_str(include_str!("../../fixtures/parity/d3-path/render.json")).unwrap()
}
pub fn figure() -> ChartResult<Plot> {
    let cases = cases();
    assert_eq!(cases.version, 1);
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [0., 1.])
        .build()?;
    let mut draft = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points().size(0.1))
        .x_axis(x_axis().visible(false))
        .y_axis(y_axis().visible(false))
        .title(title("Retained paths"));
    for case in cases.cases {
        let path = PathRequest {
            version: 1,
            digits: None,
            limits: Default::default(),
            operations: case.operations,
        }
        .build()?;
        let mut layer = vector_path(&case.id, path.geometry());
        if let Some(map) = case.transform {
            layer = layer.transform(chart_core::path::Affine::new(map)?, 0.001, 10000)?;
        }
        draft = draft.layer(
            layer
                .anchor(case.anchor.clone())
                .fill(case.fill)
                .stroke(case.stroke)
                .overflow(case.overflow),
        );
        if let Anchor::Output { x, y } = case.anchor
            && x > 0.
            && y < 250.
        {
            draft = draft.layer(
                labels()
                    .id(format!("label-{}", case.id))
                    .output_at(x - 48., y + 40.)
                    .text(case.id)
                    .style(text_style().size(0.7)),
            );
        }
    }
    draft.build()
}
