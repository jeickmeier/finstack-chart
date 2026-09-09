//! All 20 curve factories and 19 paired-boundary areas through the ordinary chart engine.
use chart_core::{ChartResult, prelude::*, shape::CurveSpec};
pub fn figure(presentation: Option<ThemeBuilder>) -> ChartResult<Plot> {
    let inventory: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/shapes/inventory.json")).unwrap();
    let names: Vec<_> = inventory["exports"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["kind"] == "curve")
        .map(|v| v["name"].as_str().unwrap().trim_start_matches("curve"))
        .collect();
    let base = Data::columns()
        .column("x", [0., 32.])
        .column("y", [0., 20.])
        .build()?;
    let mut draft = plot(base)
        .aes(aes().x("x").y("y"))
        .x_axis(
            x_axis()
                .scale(scale_linear().domain(0., 32.))
                .visible(false),
        )
        .y_axis(
            y_axis()
                .scale(scale_linear().domain(0., 20.))
                .visible(false),
        )
        .title(title(
            "Cartesian curves / projected lines and general areas",
        ));
    for (i, name) in names.iter().enumerate() {
        let curve: CurveSpec = serde_json::from_value(serde_json::json!({"kind":name})).unwrap();
        let ox = (i % 4) as f64 * 8. + 0.8;
        let oy = 16. - (i / 4) as f64 * 4.;
        let xy = [[0., 0.], [1., 2.], [2., -1.], [3., 3.], [4., 1.], [6., 2.]];
        let data = Data::columns()
            .name(format!("line-{name}"))
            .column("x", xy.map(|p| ox + p[0]))
            .column("y", xy.map(|p| oy + 2. + p[1] * 0.3))
            .build()?;
        draft = draft
            .layer(
                shape_line()
                    .name(format!("line-{name}"))
                    .data(data.clone())
                    .curve(curve)
                    .color(chart_core::color::Paint::from_css("#2162a8")?)
                    .size(1.5),
            )
            .layer(
                points()
                    .name(format!("sources-{name}"))
                    .data(data)
                    .color(chart_core::color::Paint::from_css("#d85b24")?)
                    .size(1.5),
            );
        if *name != "Bundle" {
            let data = Data::columns()
                .name(format!("area-{name}"))
                .column("x", xy.map(|p| ox + p[0]))
                .column("y", xy.map(|p| oy + 0.25 + p[1] * 0.15))
                .column("x2", xy.map(|p| ox + p[0] + p[1] * 0.2))
                .column("y2", xy.map(|p| oy + 0.9 + p[1] * 0.2))
                .build()?;
            draft = draft.layer(
                shape_area()
                    .name(format!("area-{name}"))
                    .data(data)
                    .aes(aes().x2("x2").y2("y2"))
                    .curve(curve)
                    .color(chart_core::color::Paint::from_css("rgba(33,98,168,0.55)")?),
            );
        }
        draft = draft.layer(
            labels()
                .id(format!("label-{name}"))
                .at(ox, oy + 3.6)
                .text(*name)
                .style(text_style().size(0.85)),
        );
    }
    if let Some(presentation) = presentation {
        draft = draft.theme(presentation);
    }
    draft.build()
}
