//! GG13: GeomDotplot keeps physical circles in linear coordinates and explicitly
//! disclaims nonlinear coordinates (pinned ggplot2 4.0.3 draw_group source).
use chart_core::{
    ChartResult, DiagnosticCode, Rect, ResourceId, Revision,
    grammar::{CartesianCoordinate, CoordinateSpec, RadialCoordinate, TransformedCoordinate},
    layout::{LayoutRequest, layout},
    prelude::*,
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn dotplot_linear_coordinates_keep_circles_and_nonlinear_is_explicit() {
    for spec in [
        CoordinateSpec::Cartesian(CartesianCoordinate {
            clip: chart_core::grammar::CoordinateClip::Off,
            expand: [false; 4],
            ..Default::default()
        }),
        CoordinateSpec::Cartesian(CartesianCoordinate {
            clip: chart_core::grammar::CoordinateClip::Off,
            flip: true,
            expand: [false; 4],
            ..Default::default()
        }),
        CoordinateSpec::Radial(RadialCoordinate::default()),
        CoordinateSpec::Transformed(TransformedCoordinate::default()),
    ] {
        let d = Data::columns()
            .column("x", [0., 0., 0.5, 1., 1.])
            .build()
            .unwrap();
        let p = plot(d)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x"))
            .layer(chart_core::plot::dotplot())
            .coordinate(spec.clone())
            .build()
            .unwrap();
        let mut r = LayoutRequest::new(
            Rect::new(0., 0., 600., 300.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        for a in &mut r.axes {
            a.visible = false;
        }
        let result = layout(p.chart().unwrap().prepare().unwrap(), &r, &Metrics);
        if matches!(spec, CoordinateSpec::Cartesian(_)) {
            let f = result.unwrap();
            let mut count = 0;
            for item in f.scene().items().iter().filter(|i| i.layer.is_some()) {
                if let Primitive::ShapePath { geometry, .. } = &item.primitive {
                    let points = geometry
                        .flatten(0.001, 10000)
                        .unwrap()
                        .subpaths
                        .into_iter()
                        .flat_map(|s| s.points)
                        .collect::<Vec<_>>();
                    let minx = points.iter().map(|p| p.x()).fold(f64::INFINITY, f64::min);
                    let maxx = points
                        .iter()
                        .map(|p| p.x())
                        .fold(f64::NEG_INFINITY, f64::max);
                    let miny = points.iter().map(|p| p.y()).fold(f64::INFINITY, f64::min);
                    let maxy = points
                        .iter()
                        .map(|p| p.y())
                        .fold(f64::NEG_INFINITY, f64::max);
                    // Shared circular Beziers carry a 0.01-pixel radial bound;
                    // independent extrema can differ by twice that bound.
                    assert!(((maxx - minx) - (maxy - miny)).abs() < 0.02);
                    count += 1;
                }
            }
            assert_eq!(count, 5);
        } else {
            assert_eq!(
                result.unwrap_err().code,
                DiagnosticCode::UnsupportedCapability
            );
        }
    }
}
