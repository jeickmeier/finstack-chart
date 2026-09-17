//! Coordinate raster paint and source-cell inspection share the same sampled coverage.
#[path = "../../../examples/common/ggplot_surface_recipes.rs"]
mod fixtures;
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::*,
    inspection::{InspectionMode, Inspector},
    layout::{LayoutRequest, layout},
    prelude::*,
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn warped_raster_replay_paint_and_hit_regions_exclude_the_inner_hole() {
    let original = fixtures::author(6)
        .unwrap()
        .edit()
        .coordinate(CoordinateSpec::Radial(RadialCoordinate {
            inner_radius: 0.5,
            clip: Some(CoordinateClip::On),
            expand: false,
            ..Default::default()
        }))
        .build()
        .unwrap();
    let restored = Plot::from_json(&original.to_json().unwrap()).unwrap();
    let prepared = restored.chart().unwrap().prepare().unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 240., 180.).unwrap(),
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
    let frame = Arc::new(layout(prepared, &request, &Metrics).unwrap());
    assert_eq!(frame.scene().wire_version(), 22);
    let (index, hits) = frame
        .scene()
        .items()
        .iter()
        .enumerate()
        .find_map(|(i, item)| match &item.primitive {
            Primitive::RasterImage { hits, .. } if !hits.is_empty() => Some((i, hits)),
            _ => None,
        })
        .unwrap();
    let inspector = Inspector::new(frame.clone(), 0.01, 64).unwrap();
    let plot = frame.plot().unwrap();
    let center = Point::new(
        plot.origin().x() + plot.width() / 2.,
        plot.origin().y() + plot.height() / 2.,
    )
    .unwrap();
    assert!(
        inspector
            .query(center, InspectionMode::Auto)
            .hits
            .is_empty()
    );
    for region in hits.iter().step_by((hits.len() / 16).max(1)) {
        let p = Point::new(
            region.bounds.origin().x() + region.bounds.width() / 2.,
            region.bounds.origin().y() + region.bounds.height() / 2.,
        )
        .unwrap();
        let query = inspector.query(p, InspectionMode::Auto);
        assert_eq!(query.hits.len(), 1);
        assert_eq!(
            query.hits[0].target,
            frame.targets()[index][region.target_index]
        );
    }
    assert!(inspector.semantic_targets().len() <= frame.targets()[index].len());
}
