//! FIX-GG06: reference position authoring, orientation, wire and legacy isolation.
use chart_core::plot::{ggplot_dodge, ggplot_fill, ggplot_stack, jitter_dodge};
use chart_core::{
    grammar::{DodgePreserve, Orientation, PreparedGeometry},
    prelude::*,
};
#[test]
fn positions_prepare_roundtrip_and_orientation() {
    let data = Data::columns()
        .column("x", vec![1., 1., 2.])
        .column("y", vec![2., 3., 1.])
        .column("g", vec!["a", "b", "a"])
        .build()
        .unwrap();
    for horizontal in [false, true] {
        for position in [
            ggplot_stack().vjust(0.5),
            ggplot_fill().reverse(true),
            ggplot_dodge().width(0.8).preserve(DodgePreserve::Single),
            dodge2().width(0.8).padding(0.2),
            nudge(0.2, -0.1),
            jitter_dodge(42).displacement(0.2, 0.1),
        ] {
            let p = plot(data.clone())
                .profile(chart_core::grammar::Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y").group("g"))
                .layer(points().position(position).orientation(if horizontal {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                }))
                .build()
                .unwrap();
            let wire = p.to_json().unwrap();
            let q = Plot::from_json(&wire).unwrap();
            let prepared = q.chart().unwrap().prepare().unwrap();
            assert_eq!(prepared.layers()[0].marks().len(), 3);
            assert!(
                prepared.layers()[0]
                    .marks()
                    .iter()
                    .all(|m| matches!(m.geometry, PreparedGeometry::Point(_)))
            );
        }
    }
}
#[test]
fn positioned_categories_project_without_changing_identity() {
    use chart_core::{
        Rect, ResourceId, Revision,
        layout::{LayoutRequest, layout},
        services::{
            ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units,
        },
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    for cats in [vec!["a", "a"], vec!["a", "b"]] {
        let d = Data::columns()
            .column("x", categorical(cats))
            .column("g", vec!["a", "b"])
            .build()
            .unwrap();
        for pos in [
            nudge(0.2, 0.),
            ggplot_dodge().width(0.8),
            dodge2().width(0.8),
            jitter_dodge(42),
        ] {
            let p = plot(d.clone())
                .profile(chart_core::grammar::Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).group("g"))
                .layer(points().position(pos))
                .build()
                .unwrap();
            let mut request = LayoutRequest::new(
                Rect::new(0., 0., 400., 300.).unwrap(),
                Units::Points,
                ResourceDescriptor {
                    id: ResourceId::new(1),
                    revision: Revision::INITIAL,
                    kind: ResourceKind::Font,
                    byte_len: 1,
                },
            );
            for axis in &mut request.axes {
                axis.visible = false;
            }
            let result = layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics);
            assert!(result.is_ok(), "{result:?}");
        }
    }
}
#[test]
fn jitter_dodge_retained_and_fresh_data_identity_match() {
    let batch = || {
        Data::columns()
            .identity(901)
            .keys([11, 12, 13])
            .column("x", [1., 1., 2.])
            .column("y", [2., 3., 1.])
            .column("g", ["a", "b", "a"])
            .build()
            .unwrap()
    };
    let author = |d: Data| {
        plot(d)
            .profile(chart_core::grammar::Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y").group("g"))
            .layer(points().position(jitter_dodge(42)))
            .build()
            .unwrap()
    };
    let retained = author(batch());
    let fresh = author(batch());
    let restored = Plot::from_json(&retained.to_json().unwrap()).unwrap();
    let geometries = |p: &Plot| {
        p.chart().unwrap().prepare().unwrap().layers()[0]
            .marks()
            .iter()
            .map(|m| m.geometry.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(geometries(&retained), geometries(&fresh));
    assert_eq!(geometries(&retained), geometries(&restored));
}
#[test]
fn single_preservation_uses_all_facet_populations() {
    let data = Data::columns()
        .column("lo", [0.6, 0.6, 0.6])
        .column("hi", [1.4, 1.4, 1.4])
        .column("y", [1., 2., 3.])
        .column("g", ["a", "b", "a"])
        .column("panel", ["one", "one", "two"])
        .build()
        .unwrap();
    for pos in [
        ggplot_dodge().width(0.8).preserve(DodgePreserve::Single),
        dodge2().width(0.8).preserve(DodgePreserve::Single),
    ] {
        let p = plot(data.clone())
            .profile(chart_core::grammar::Profile::Ggplot2_4_0_3)
            .aes(aes().x("lo").x2("hi").y("y").y2(0.).group("g"))
            .layer(rectangle().position(pos))
            .facet(facet_wrap("panel"))
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        assert_eq!(prepared.panels().len(), 2);
        let only = &prepared.panels()[1].chart.layers()[0].marks()[0];
        let PreparedGeometry::Rectangle { from, to } = only.geometry else {
            panic!("rectangle expected")
        };
        assert!(((to.x() - from.x()).abs() - 0.4).abs() < 1e-12, "{only:?}");
    }
}
