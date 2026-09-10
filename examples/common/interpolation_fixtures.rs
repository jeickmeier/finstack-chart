//! ITP-07: one explicit sampled state used by native and immutable publication.
use chart_core::{
    ChartResult, Revision,
    composition::Anchor,
    grammar::OperationRef,
    interpolate::{
        InterpolationFactory, InterpolationSpec, Interpolator, Number, TransformSyntax, Value,
        ZoomView,
    },
    path::{Affine, PathRequest},
    prelude::*,
    scales::{ScaleConstructor, ScaleOptions, ScaleTraining},
    scene::{Color, Stroke},
    theme::NamedTheme,
};
pub fn factory(native: bool, mode: &str) -> InterpolationFactory {
    InterpolationFactory::registered(
        OperationRef::new(
            if native {
                "example.native_interpolation"
            } else {
                "example.interpolation"
            },
            Revision::new(1),
        ),
        serde_json::json!({"mode":mode}),
    )
}
pub fn motion(t: f64) -> ChartResult<(Affine, [f64; 3])> {
    let transform = Interpolator::new(InterpolationSpec::TransformText {
        a: "translate(0,0) rotate(-25) scale(0.7)".into(),
        b: "translate(40,10) rotate(65) scale(1.3)".into(),
        syntax: TransformSyntax::Svg,
    })?
    .sample_transform(t)?;
    let zoom = Interpolator::new(InterpolationSpec::Zoom {
        a: ZoomView::new([0., 0., 100.])?,
        b: ZoomView::new([40., 20., 50.])?,
        rho: None,
    })?;
    let Value::Array(values) = zoom.sample(t)? else {
        unreachable!()
    };
    let values = values
        .into_iter()
        .map(|v| {
            if let Value::Number(Number(n)) = v {
                n
            } else {
                unreachable!()
            }
        })
        .collect::<Vec<_>>();
    Ok((transform, values.try_into().unwrap()))
}
pub fn figure(t: f64, native: bool) -> ChartResult<Plot> {
    let registry = chart_extension_example::registry()?;
    let color = ScaleConstructor::Linear.create_with_registry(
        ScaleOptions {
            factory: Some(factory(native, "LabColor")),
            range: Some(
                ["#d54a3a", "#28689b"]
                    .map(|v| Value::Text(v.into()))
                    .to_vec(),
            ),
            ..Default::default()
        },
        &registry,
    )?;
    let size = ScaleConstructor::Linear.create_with_registry(
        ScaleOptions {
            factory: Some(factory(native, "SquaredNumber")),
            range: Some([5., 13.].map(Value::number).to_vec()),
            ..Default::default()
        },
        &registry,
    )?;
    let data = Data::columns()
        .name("interpolation")
        .keys((0..=20).map(|i| 9007199254741001 + i))
        .column(
            "x",
            (0..=20).map(|i| f64::from(i) / 20.).collect::<Vec<_>>(),
        )
        .build()?;
    let (transform, zoom) = motion(t)?;
    let triangle = PathRequest {
        version: 1,
        digits: None,
        limits: Default::default(),
        operations: vec![
            PathOp::MoveTo([-24., 18.]),
            PathOp::LineTo([0., -25.]),
            PathOp::LineTo([24., 18.]),
            PathOp::ClosePath,
        ],
    }
    .build()?;
    let camera = PathRequest {
        version: 1,
        digits: None,
        limits: Default::default(),
        operations: vec![PathOp::Rect([-35., -20., 70., 40.])],
    }
    .build()?;
    plot(data)
        .extensions(registry)
        .aes(aes().x("x").y(1.).color("x").color_scale("perceptual"))
        .layer(points().numeric_scale(
            NumericAesthetic::Size,
            "x",
            size.mapped(ScaleTraining::Authored)?,
        ))
        .scale(color_mapped(
            "perceptual",
            color.mapped(ScaleTraining::Authored)?,
        ))
        .legend(legend().scale("perceptual").title("Registered Lab ramp"))
        .x_axis(
            x_axis()
                .scale(scale_linear().domain(0., 1.))
                .range(65., 415.),
        )
        .y_axis(
            y_axis()
                .scale(scale_linear().domain(0., 2.))
                .range(155., 65.)
                .visible(false),
        )
        .layer(
            vector_path("sampled-transform", triangle.geometry())
                .transform(transform, 0.001, 10000)?
                .anchor(Anchor::Output { x: 155., y: 255. })
                .fill(Some(Color {
                    red: 213,
                    green: 74,
                    blue: 58,
                    alpha: 220,
                })),
        )
        .layer(
            vector_path("sampled-zoom", camera.geometry())
                .transform(
                    Affine::new([100. / zoom[2], 0., 0., 100. / zoom[2], -zoom[0], -zoom[1]])?,
                    0.001,
                    10000,
                )?
                .anchor(Anchor::Output { x: 395., y: 255. })
                .fill::<Color>(None)
                .stroke(Some(Stroke {
                    color: Color {
                        red: 40,
                        green: 104,
                        blue: 155,
                        alpha: 255,
                    },
                    width: 2.,
                })),
        )
        .layer(
            labels()
                .id("transform-label")
                .output_at(135., 330.)
                .text("Sampled transform"),
        )
        .layer(
            labels()
                .id("zoom-label")
                .output_at(340., 330.)
                .text("Sampled zoom"),
        )
        .title(title(format!("Shared interpolation / t = {t:.2}")))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()
}
