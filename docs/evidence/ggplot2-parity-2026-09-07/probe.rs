use chart_core::{
    grammar::*, layout::*, portable::Session, scales::*, scene::*, services::*, state::ChartState,
    *,
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.chars().count() as f64 * 6., 9., 3.)
    }
}
fn main() {
    let s = Session::new(
        include_str!("../../../fixtures/bindings/chart.json"),
        include_str!("../../../fixtures/bindings/data.json"),
    )
    .unwrap();
    let layer = Layer::new(
        LayerId::new(1),
        DatasetId::new(9007199254741001),
        Geom::Point,
        SourceAes::new().x(FieldId::new(1)).y(FieldId::new(2)),
    )
    .colored(ColorEncoding {
        title: Some("PARITY_LEGEND".into()),
        id: ScaleId::new(5),
        input: ColorInput::Category(FieldId::new(8)),
        scale: ColorScale::Discrete {
            domain: None,
            palette: vec![
                chart_core::theme::rgb(255, 0, 0),
                chart_core::theme::rgb(0, 0, 255),
            ],
            missing: chart_core::theme::rgb(0, 0, 0),
        },
    });
    let mut d = ChartDefinition::new(Revision::INITIAL).layer(layer);
    let prepare = |d: &ChartDefinition| {
        Compiler::new().prepare(
            d,
            &s.source(),
            &ChartState::default(),
            CompileLimits::default(),
        )
    };
    let p = prepare(&d).unwrap();
    let legend = p.layers()[0].color_legend().unwrap();
    let facet_keys = legend
        .entries
        .iter()
        .map(|(label, _)| PanelKey {
            values: vec![GroupValue::Text(label.clone())],
        })
        .collect();
    println!(
        "Prepared legend: {:?}, entries={}",
        legend.title,
        legend.entries.len()
    );
    let font = ResourceDescriptor {
        id: ResourceId::new(1),
        revision: Revision::INITIAL,
        kind: ResourceKind::Font,
        byte_len: 10,
    };
    let request = LayoutRequest::new(Rect::new(0., 0., 640., 480.).unwrap(), Units::Points, font);
    let scene = layout(Arc::new(p), &request, &Metrics).unwrap();
    let count = scene
        .scene()
        .items()
        .iter()
        .filter(|i| matches!(&i.primitive, Primitive::Text {text,..} if text == "PARITY_LEGEND"))
        .count();
    println!("Non-faceted legend title scene items: {count}");
    assert_eq!(count, 0);
    let mut faceted = d.clone();
    faceted.facets = Some(FacetSpec {
        fields: vec![FieldId::new(8)],
        order: facet_keys,
        layout: FacetLayout::Wrap { columns: 2 },
        empty: EmptyPanels::Keep,
        scales: FacetScales::default(),
        gap: 8.,
        collect_guides: true,
    });
    let control = layout(Arc::new(prepare(&faceted).unwrap()), &request, &Metrics).unwrap();
    let count = control
        .scene()
        .items()
        .iter()
        .filter(|i| matches!(&i.primitive, Primitive::Text {text,..} if text == "PARITY_LEGEND"))
        .count();
    println!("Faceted positive-control legend title scene items: {count}");
    assert_eq!(count, 1);
    d.layers[0].geom = Geom::line();
    assert_eq!(
        prepare(&d).unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    println!(
        "Category color without explicit group: {:?}",
        prepare(&d).err().map(|e| (e.code, e.message))
    );
    if let Mappings::Source(aes) = &mut d.layers[0].mappings {
        aes.group = Some(FieldId::new(8));
    }
    println!("With explicit group: success={}", prepare(&d).is_ok());
    assert!(prepare(&d).is_ok());
}
