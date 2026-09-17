//! GG13 measured coordinate guides and immutable public author qualification.
use chart_core::{
    Rect, ResourceId, Revision,
    layout::{LayoutRequest, layout},
    prelude::*,
    scene::GuideRole,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn shape(
        &self,
        r: chart_core::typography::ShapeRequest<'_>,
    ) -> ChartResult<chart_core::typography::ShapedRun> {
        Ok(chart_core::typography::ShapedRun {
            text: r.run.text.clone(),
            font: *r.default_font,
            font_size: r.font_size,
            language: r.run.language.clone(),
            direction: r.run.direction,
            tabular: r.run.tabular,
            metrics: TextMetrics::new(r.run.text.len() as f64 * 5., 8., 2.)?,
            glyphs: vec![],
            outlines: vec![],
            used_fallback: false,
        })
    }
    fn measure(&self, r: TextRequest) -> chart_core::ChartResult<TextMetrics> {
        TextMetrics::new(r.text.chars().count() as f64 * 6., 8., 2.)
    }
}

#[path = "../../../examples/common/ggplot_coordinate_controls.rs"]
mod fixtures;
fn draw(p: &Plot) -> chart_core::layout::LaidOutChart {
    let request = LayoutRequest::new(
        Rect::new(0., 0., 600., 400.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap()
}
#[test]
fn public_radial_guides_seam_inner_axis_and_replay() {
    for mode in 0..6 {
        let p = fixtures::author(mode).unwrap();
        let frame = draw(&p);
        let restored = Plot::from_json(&p.to_json().unwrap()).unwrap();
        let replay = draw(&restored);
        assert_eq!(frame.scene().items(), replay.scene().items());
        let labels = frame
            .scene()
            .items()
            .iter()
            .filter_map(|item| item.guide.as_ref())
            .filter(|g| g.role == GuideRole::Label)
            .collect::<Vec<_>>();
        assert!(!labels.is_empty(), "mode {mode}");
        if matches!(mode, 0 | 2 | 3 | 4) {
            assert!(
                labels.iter().any(|g| g.label.as_deref() == Some("0/4")),
                "mode {mode}: {labels:?}"
            );
        }
        if mode == 4 {
            let inner = p.guide("theta-inner").unwrap().id();
            assert!(labels.iter().any(|g| g.guide == inner));
        }
        let plot = frame.plot().unwrap();
        if mode == 5 {
            assert!((plot.width() - plot.height()).abs() < 1e-10);
        }
        if mode == 1 {
            assert!((plot.height() / plot.width() - 0.8535533905932737).abs() < 1e-12);
        }
    }
}

#[test]
fn radial_minor_ticks_keep_roles_and_cartesian_flip_moves_guides() {
    use chart_core::{
        grammar::*,
        interpolate::Number,
        layout::{AxisSide, GgplotAxisOptions, MinorBreaks},
    };
    let data = Data::columns()
        .column("x", vec![0., 4.])
        .column("y", vec![0., 4.])
        .build()
        .unwrap();
    let p = plot(data.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale_linear().domain(0., 4.))
                .minor_breaks(Some(MinorBreaks::Numeric(vec![
                    Number(0.5),
                    Number(1.5),
                    Number(2.5),
                    Number(3.5),
                ])))
                .ggplot_axis(Some(GgplotAxisOptions {
                    minor_ticks: true,
                    ..Default::default()
                })),
        )
        .coordinate(CoordinateSpec::Radial(RadialCoordinate {
            expand: false,
            ..Default::default()
        }))
        .build()
        .unwrap();
    let frame = draw(&p);
    let minor = frame
        .scene()
        .items()
        .iter()
        .filter_map(|item| item.guide.as_ref())
        .filter(|g| g.scope == ["minor"])
        .collect::<Vec<_>>();
    assert_eq!(minor.len(), 4);
    assert!(
        minor
            .iter()
            .all(|g| g.role == GuideRole::Line && g.tick.is_some())
    );
    let p = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .coordinate(CoordinateSpec::Cartesian(CartesianCoordinate {
            flip: true,
            ..Default::default()
        }))
        .build()
        .unwrap();
    let frame = draw(&p);
    let x = p.axis("x").unwrap().guide().id();
    let y = p.axis("y").unwrap().guide().id();
    assert_eq!(frame.guides()[&x].spec.side, AxisSide::Left);
    assert_eq!(frame.guides()[&y].spec.side, AxisSide::Bottom);
}

#[test]
fn reference_fixed_ratio_rejects_free_facets_but_preserves_legacy() {
    use chart_core::grammar::*;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/coordinate-guide-controls.json"
    ))
    .unwrap();
    assert_eq!(fixture["facet_errors"].as_array().unwrap().len(), 6);
    assert!(
        fixture["facet_errors"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["result"]["ok"] == false)
    );
    let data = Data::columns()
        .column("x", vec![0., 1., 0., 4.])
        .column("y", vec![0., 1., 0., 2.])
        .column("g", vec!["a", "a", "b", "b"])
        .build()
        .unwrap();
    let request = LayoutRequest::new(
        Rect::new(0., 0., 600., 400.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for profile in [Profile::Ggplot2_4_0_3, Profile::LibraryV1] {
        let p = plot(data.clone())
            .profile(profile)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .facet(facet_wrap("g").free_x(true))
            .coordinate(CoordinateSpec::Cartesian(CartesianCoordinate {
                ratio: Some(1.),
                ..Default::default()
            }))
            .build()
            .unwrap();
        let result = layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics);
        assert_eq!(result.is_err(), profile == Profile::Ggplot2_4_0_3);
    }
}

#[test]
fn coordinate_guide_animation_uses_relocated_geometry_and_rejects_radial() {
    use chart_core::{
        DiagnosticCode, Limits,
        grammar::{CartesianCoordinate, CoordinateSpec, RadialCoordinate, TransformedCoordinate},
        layout::LayoutGuideTransition,
    };
    use std::sync::Arc;
    for spec in [
        CoordinateSpec::Cartesian(CartesianCoordinate {
            flip: true,
            expand: [false; 4],
            ..Default::default()
        }),
        CoordinateSpec::Transformed(TransformedCoordinate {
            y: chart_core::scales::GgplotTransform::Sqrt,
            view: CartesianCoordinate {
                expand: [false; 4],
                ..Default::default()
            },
            ..Default::default()
        }),
        CoordinateSpec::Radial(RadialCoordinate::default()),
    ] {
        let d = Data::columns()
            .column("x", [0., 0.25, 0.5, 0.75, 1.])
            .column("y", [0., 0.25, 0.5, 0.75, 1.])
            .build()
            .unwrap();
        let p = plot(d)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .coordinate(spec.clone())
            .build()
            .unwrap();
        let f = Arc::new(draw(&p));
        let transition = LayoutGuideTransition::new(f.clone(), f.clone(), Limits::default());
        if matches!(spec, CoordinateSpec::Radial(_)) {
            assert_eq!(
                transition.unwrap_err().code,
                DiagnosticCode::UnsupportedCapability
            );
            assert!(f.guide_presentation().is_empty());
            assert!(!f.guide_snapshots().is_empty());
        } else {
            let transition = transition.unwrap();
            for fraction in [0., 0.5, 1.] {
                let sampled = transition.sample(fraction).unwrap();
                for item in sampled.scene().items() {
                    if let Some(component) = &item.guide
                        && matches!(component.role, GuideRole::Domain | GuideRole::Line)
                    {
                        let original = f
                            .scene()
                            .items()
                            .iter()
                            .find(|i| {
                                i.guide.as_ref().is_some_and(|g| {
                                    g.guide == component.guide
                                        && g.role == component.role
                                        && g.index == component.index
                                })
                            })
                            .unwrap();
                        assert_eq!(item.primitive, original.primitive, "{spec:?} at {fraction}");
                    }
                }
            }
        }
    }
}
