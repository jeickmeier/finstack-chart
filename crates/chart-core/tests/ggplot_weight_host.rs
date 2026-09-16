//! GG-06 follow-up: owned fields and expressions take the canonical weight path.
use chart_core::{
    grammar::PreparedRows,
    plot::host::{Component, Draft},
    prelude::*,
};

#[test]
fn host_weight_fields_and_expressions_preserve_signed_values() {
    let data = Data::columns()
        .column("x", [0.5, 1.5])
        .column("w", [2., -3.])
        .build()
        .unwrap();
    let mappings = Component::new("aes", "[]")
        .unwrap()
        .set("x", r#"["x"]"#)
        .unwrap();
    for kind in ["bin", "count"] {
        let mut stat = Component::new(kind, "[]")
            .unwrap()
            .set("x", r#"["x"]"#)
            .unwrap();
        let method = if kind == "bin" {
            stat = stat
                .set("breaks", "[[0,1,2]]")
                .unwrap()
                .set("ggplot_bin", "[{}]")
                .unwrap();
            "bin_weight"
        } else {
            stat = stat.set("ggplot_count", "[]").unwrap();
            "count_weight"
        };
        let field = stat.field(method, data.field("w").unwrap()).unwrap();
        let expression = stat
            .with(
                method,
                &Component::source_expression(data.field("w").unwrap()),
            )
            .unwrap();
        for weighted in [field, expression] {
            let layer = Component::new("points", "[]")
                .unwrap()
                .with("stat", &weighted)
                .unwrap();
            let plot = Draft::new(&data)
                .with("aes", &mappings)
                .unwrap()
                .with("layer", &layer)
                .unwrap()
                .build()
                .unwrap();
            let prepared = plot.chart().unwrap().prepare().unwrap();
            let values = match prepared.layers()[0].table().rows() {
                PreparedRows::Binned(rows) => rows
                    .iter()
                    .map(|r| r.statistics.as_ref().unwrap().count)
                    .collect::<Vec<_>>(),
                PreparedRows::Statistical(rows) => rows
                    .iter()
                    .map(|r| {
                        r.value(&chart_core::grammar::StatField::WeightedCount)
                            .unwrap()
                    })
                    .collect(),
                _ => panic!("expected generated weight output"),
            };
            assert_eq!(values, [2., -3.]);
        }
    }
}
