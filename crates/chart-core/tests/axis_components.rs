//! AXIS-05 portable roles and independent component styling.
use chart_core::{
    ChartResult, DiagnosticCode, Rect, ResourceId, Revision,
    grammar::Compiler,
    layout::{GuideComponents, GuideProfile, LayoutRequest, layout},
    prelude::*,
    scene::{GuideRole, Primitive},
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    state::ChartState,
};
use std::{cell::Cell, sync::Arc};
struct Metrics(Cell<usize>);
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        self.0.set(self.0.get() + 1);
        TextMetrics::new(
            r.text.len() as f64 * r.font_size * 0.5,
            r.font_size * 0.8,
            r.font_size * 0.2,
        )
    }
}
fn options() -> GuideComponents {
    serde_json::from_value(serde_json::json!({
        "domain":{"color":"#1122cc","width":2.,"dashes":[4.,2.]},
        "ticks":{"color":"#008844","width":1.5,"dashes":[2.,2.]},
        "labels":{"color":"#cc2211","font_size":16.},
        "per_tick":[
            {"index":1,"line":{"visible":false},"label":{"color":"#0022ff","font_size":20.}},
            {"index":2,"visible":false},
            {"index":3,"line":{"width":3.,"dashes":[]},"label":{"visible":false}}
        ]
    }))
    .unwrap()
}
fn plot_with(components: GuideComponents) -> Plot {
    plot(
        Data::columns()
            .column("x", [0., 0.5, 1.])
            .column("y", [0., 1., 0.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(
        x_axis()
            .scale(scale_linear().domain(0., 1.))
            .range(50., 450.)
            .guide_profile(GuideProfile::D3_3_0_0)
            .tick_size_inner(-30.)
            .tick_values(Some(vec![0f64.into(), 0.5.into(), 0.5.into(), 1f64.into()]))
            .tick_format(Some(chart_core::layout::GuideFormatter::Labels(
                ["start", "same", "same", "end"].map(str::to_owned).to_vec(),
            )))
            .guide_components(Some(components)),
    )
    .y_axis(y_axis().visible(false))
    .build()
    .unwrap()
}
fn request() -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 500., 300.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
fn prepared(p: &Plot) -> Arc<chart_core::grammar::PreparedChart> {
    Arc::new(
        Compiler::with_extensions(p.extensions().clone())
            .prepare(
                p.definition(),
                &p.source(),
                &ChartState::default(),
                p.compile_limits(),
            )
            .unwrap(),
    )
}
#[test]
fn component_cascade_keeps_semantics_and_uses_resolved_font_metrics_without_fake_targets() {
    let p = plot_with(options());
    assert_eq!(p.definition().wire_version(), 14);
    let wire = p.to_json().unwrap();
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    let c = layout(prepared(&p), &request(), &Metrics(Cell::new(0))).unwrap();
    let ticks = &c
        .guides()
        .values()
        .find(|g| g.spec.side == chart_core::layout::AxisSide::Bottom)
        .unwrap()
        .ticks;
    assert_eq!(ticks.len(), 4);
    let components: Vec<_> = c
        .scene()
        .items()
        .iter()
        .enumerate()
        .filter(|(_, i)| i.guide.is_some())
        .collect();
    assert_eq!(components.len(), 5);
    assert_eq!(c.scene().wire_version(), 14);
    for (index, _) in &components {
        assert!(c.targets()[*index].is_empty());
    }
    let domain = &components[0].1;
    assert_eq!(domain.guide.as_ref().unwrap().role, GuideRole::Domain);
    assert!(
        matches!(&domain.primitive,Primitive::DashedPath{stroke,dashes,..} if stroke.width==2. && stroke.color.blue==204 && dashes==&[4.,2.])
    );
    let line0 = components
        .iter()
        .find(|(_, i)| {
            i.guide
                .as_ref()
                .is_some_and(|g| g.role == GuideRole::Line && g.index == Some(0))
        })
        .unwrap()
        .1;
    assert!(
        matches!(&line0.primitive,Primitive::DashedPath{stroke,dashes,..} if stroke.width==1.5 && stroke.color.green==136 && dashes==&[2.,2.])
    );
    let label1 = components
        .iter()
        .find(|(_, i)| {
            i.guide
                .as_ref()
                .is_some_and(|g| g.role == GuideRole::Label && g.index == Some(1))
        })
        .unwrap()
        .1;
    let metadata = label1.guide.as_ref().unwrap();
    assert_eq!(metadata.tick.as_ref().unwrap().occurrence, 0);
    assert_eq!(metadata.label.as_deref(), Some("same"));
    match label1.primitive {
        Primitive::Text {
            origin,
            font_size,
            color,
            ..
        } => {
            assert_eq!(font_size, 20.);
            assert_eq!(color.blue, 255);
            assert!((origin.x() - (250.5 - 20.)).abs() < 1e-9);
            assert!((origin.y() - (c.plot().unwrap().max_y() + 3. + 0.71 * 20.)).abs() < 1e-9);
        }
        _ => panic!("plain resolved label"),
    }
    let line3 = components
        .iter()
        .find(|(_, i)| {
            i.guide
                .as_ref()
                .is_some_and(|g| g.role == GuideRole::Line && g.index == Some(3))
        })
        .unwrap()
        .1;
    assert!(matches!(&line3.primitive,Primitive::Rule{stroke,..} if stroke.width==3.));
    assert!(
        !components
            .iter()
            .any(|(_, i)| i.guide.as_ref().unwrap().index == Some(2))
    );
}
#[test]
fn duplicate_occurrences_blank_labels_and_style_failures_are_explicit_and_bounded() {
    let mut styles = options();
    styles.per_tick.clear();
    let p = plot_with(styles.clone());
    let c = layout(prepared(&p), &request(), &Metrics(Cell::new(0))).unwrap();
    let line_keys: Vec<_> = c
        .scene()
        .items()
        .iter()
        .filter_map(|i| i.guide.as_ref())
        .filter(|g| g.role == GuideRole::Line)
        .map(|g| g.tick.as_ref().unwrap().occurrence)
        .collect();
    assert_eq!(line_keys, [0, 0, 1, 0]);
    for bad in [
        serde_json::json!({"ticks":{"width":0.}}),
        serde_json::json!({"domain":{"dashes":[1.,2.,3.]}}),
        serde_json::json!({"labels":{"font_size":-1.}}),
        serde_json::json!({"per_tick":[{"index":0},{"index":0}]}),
    ] {
        let components: GuideComponents = serde_json::from_value(bad).unwrap();
        assert!(
            p.edit()
                .x_axis(x_axis().guide_components(Some(components)))
                .build()
                .is_err()
        );
    }
    let mut r = request();
    r.limits.max_text_bytes = 20;
    let metrics = Metrics(Cell::new(0));
    assert_eq!(
        layout(prepared(&p), &r, &metrics).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(metrics.0.get(), 0);
    assert_eq!(c.scene().wire_version(), 14);
    styles.domain.visible = Some(false);
    styles.ticks.visible = Some(false);
    styles.labels.visible = Some(false);
    let hidden = plot_with(styles);
    let hidden = layout(prepared(&hidden), &request(), &Metrics(Cell::new(0))).unwrap();
    assert!(!hidden.scene().items().iter().any(|i| i.guide.is_some()));
    assert_eq!(hidden.guides().values().next().unwrap().ticks.len(), 4);
}

#[test]
fn signed_zero_ticks_have_distinct_occurrences_and_labels() {
    let p = plot_with(GuideComponents::default())
        .edit()
        .x_axis(
            x_axis()
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_values(Some(vec![(-0f64).into(), 0f64.into()]))
                .tick_format(Some(chart_core::layout::GuideFormatter::Labels(vec![
                    "minus".into(),
                    "plus".into(),
                ]))),
        )
        .build()
        .unwrap();
    let c = layout(prepared(&p), &request(), &Metrics(Cell::new(0))).unwrap();
    let keys: Vec<_> = c
        .scene()
        .items()
        .iter()
        .filter_map(|i| i.guide.as_ref())
        .filter(|g| g.role == GuideRole::Line)
        .map(|g| {
            (
                g.tick.as_ref().unwrap().occurrence,
                g.label.as_deref().unwrap(),
            )
        })
        .collect();
    assert_eq!(keys, [(0, "minus"), (1, "plus")]);
}
