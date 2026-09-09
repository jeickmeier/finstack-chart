//! FIX-S01: shape reference command streams through the shared checked path engine.
use chart_core::{
    ChartResult,
    composition::Anchor,
    path::{Affine, Command, PathGeometry, PathRequest, PathSink},
    prelude::*,
    scene::{Color, Stroke},
};
#[derive(Default)]
struct ExternalSink(Vec<Command>);
impl PathSink for ExternalSink {
    fn command(&mut self, command: &Command) -> ChartResult<()> {
        self.0.push(*command);
        Ok(())
    }
}

pub fn figure() -> ChartResult<Plot> {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/shapes/cases.json")).unwrap();
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [0., 1.])
        .build()?;
    let mut draft = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points().size(0.1))
        .x_axis(x_axis().visible(false))
        .y_axis(y_axis().visible(false))
        .title(title("Shape contexts / shared path foundation"));
    for (id, x, scale, filled, label) in [
        ("arc-quarter", 85., 5., true, "Circular sector"),
        ("arc-hole", 235., 5., true, "Annular hole"),
        (
            "line-curveBasis-6",
            370.,
            14.,
            false,
            "External sink / Bezier",
        ),
    ] {
        let case = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["id"] == id)
            .unwrap();
        let path = PathRequest {
            version: 1,
            digits: Some(3.),
            limits: Default::default(),
            operations: serde_json::from_value(case["operations"].clone()).unwrap(),
        }
        .build()?;
        let mut sink = ExternalSink::default();
        path.replay(&mut sink)?;
        let replayed = PathGeometry::from_commands(sink.0, 10_000)?;
        assert_eq!(replayed, path.geometry());
        let color = Color {
            red: 35,
            green: 97,
            blue: 166,
            alpha: 255,
        };
        let layer = vector_path(id, replayed)
            .transform(Affine::new([scale, 0., 0., scale, 0., 0.])?, 0.001, 10_000)?
            .anchor(Anchor::Output { x, y: 140. })
            .fill(filled.then_some(color))
            .stroke(Some(Stroke { color, width: 1.5 }));
        draft = draft.layer(layer).layer(
            labels()
                .id(format!("label-{id}"))
                .output_at(x - 40., 220.)
                .text(label)
                .style(text_style().size(0.85)),
        );
    }
    draft.build()
}
