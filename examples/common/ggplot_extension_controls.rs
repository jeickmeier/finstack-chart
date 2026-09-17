//! GG16 primary vector, registered key and materialization/recipe authors.
use chart_core::{
    Revision,
    grammar::*,
    plot::{AuthoringSelection, autoplot},
    prelude::*,
};
pub const CASES: usize = 10;
pub fn author(mode: usize) -> ChartResult<Plot> {
    let registry = chart_extension_example::registry()?;
    if mode == 8 {
        return plot(
            Data::columns()
                .column("x", [0., 1., 2.])
                .column("y", [2., 5., 10.])
                .column("w", [1., 2., 1.])
                .build()?,
        )
        .extensions(registry)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(
            smooth().stat(
                model_stat(ModelOptions {
                    method: ModelMethod::Registered {
                        operation: OperationRef::new(
                            chart_extension_example::models::PRESCRIBED_SLOPE,
                            Revision::new(1),
                        ),
                        parameters: serde_json::json!({"slope":2.,"envelope":1.25}),
                    },
                    xseq: Some(vec![-1., 0., 1., 2., 3.]),
                    level: 0.8,
                    ..Default::default()
                })
                .x("x")
                .y("y")
                .weight("w"),
            ),
        )
        .build();
    }

    if mode == 4 {
        let operation = OperationRef::new(
            chart_extension_example::recipe_dispatch::XY,
            Revision::new(1),
        );
        let data = registry.materialize(
            &operation,
            &serde_json::json!({"x":[0.,1.,2.],"y":[2.,4.,3.]}),
            Default::default(),
            true,
        )?;
        return autoplot(
            data,
            AuthoringSelection {
                operation,
                parameters: serde_json::json!({"line":true}),
            },
            registry,
            true,
        )?
        .build();
    }
    let x = vec![-3., -2., -1., 0., 1., 2., 3.];
    let spec = match mode {
        1 => CutSpec::Number { bins: 3 },
        2 => CutSpec::Width {
            width: 2.,
            center: Some(0.),
            boundary: None,
        },
        _ => CutSpec::Interval { bins: 3 },
    };
    let groups = cut(
        &x.iter().copied().map(Some).collect::<Vec<_>>(),
        spec,
        CutOptions::default(),
    )?;
    let data = Data::columns()
        .column("x", x)
        .column("y", [1., 4., 2., 5., 3., 6., 4.])
        .column("group", groups.column())
        .build()?;
    let layer = if mode == 3 {
        points().key_glyph(
            chart_extension_example::key_glyphs::DIAMOND,
            Revision::new(1),
            serde_json::json!({"padding":0.1}),
        )
    } else {
        points()
    };
    let builder = plot(data)
        .extensions(registry)
        .profile(Profile::Ggplot2_4_0_3)
        .scale(color_discrete("groups").domain(groups.levels.clone()))
        .aes(aes().x("x").y("y").color("group").color_scale("groups"))
        .layer(layer);
    let builder = if mode == 5 {
        builder.legend(legend().scale("groups").registered(
            chart_extension_example::guide_drawing::STRIP,
            Revision::new(1),
            serde_json::json!({}),
        ))
    } else {
        builder
    };
    let builder = if mode == 6 {
        builder.facet(facet_wrap("group").registered(
            chart_extension_example::facet_planner::REVERSE,
            Revision::new(1),
            serde_json::json!({"columns":2}),
        ))
    } else {
        builder
    };
    let builder = if mode == 7 {
        builder
            .registered_coordinate(
                chart_extension_example::coordinates::WAVE,
                Revision::new(1),
                serde_json::json!({"amplitude":0.12}),
            )
            .layer(line())
            .layer(
                line()
                    .independent()
                    .data(
                        Data::columns()
                            .name("wave-span")
                            .column("x", [-3., 3.])
                            .column("y", [3.5, 3.5])
                            .build()?,
                    )
                    .aes(aes().x("x").y("y")),
            )
    } else {
        builder
    };
    let builder = if mode == 9 {
        builder.facet(facet_wrap("group").reference(FacetPolicy {
            labeller: FacetLabeller {
                registered: Some(FacetLabelOperation {
                    operation: OperationRef::new(
                        chart_extension_example::facet_labels::LABELS,
                        Revision::new(1),
                    ),
                    parameters: serde_json::json!("context"),
                }),
                ..Default::default()
            },
            ..Default::default()
        }))
    } else {
        builder
    };
    builder.build()
}
