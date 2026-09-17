//! Combined registered protocols recompute from one coherent source snapshot.
use chart_core::{
    grammar::*, layout::*, prelude::*, scene::Primitive, services::*, transaction::CommitOutcome, *,
};
type Row = (u64, f64, f64, &'static str);
fn data(rows: &[Row]) -> Data {
    Data::columns()
        .column("x", rows.iter().map(|r| r.1).collect::<Vec<_>>())
        .column("y", rows.iter().map(|r| r.2).collect::<Vec<_>>())
        .column("g", rows.iter().map(|r| r.3).collect::<Vec<_>>())
        .keys(rows.iter().map(|r| r.0))
        .build()
        .unwrap()
}
fn author(d: Data) -> Plot {
    plot(d)
        .extensions(chart_extension_example::registry().unwrap())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").color("g").color_scale("groups"))
        .scale(color_discrete("groups"))
        .layer(points().key_glyph(
            chart_extension_example::key_glyphs::DIAMOND,
            Revision::new(1),
            serde_json::json!({"padding":0.1}),
        ))
        .legend(legend().scale("groups").registered(
            chart_extension_example::guide_drawing::STRIP,
            Revision::new(1),
            serde_json::json!({}),
        ))
        .facet(facet_wrap("g").registered(
            chart_extension_example::facet_planner::REVERSE,
            Revision::new(1),
            serde_json::json!({"columns":2}),
        ))
        .registered_coordinate(
            chart_extension_example::coordinates::WAVE,
            Revision::new(1),
            serde_json::json!({"amplitude":0.1}),
        )
        .build()
        .unwrap()
}
fn primitives(prepared: std::sync::Arc<PreparedChart>) -> Vec<Primitive> {
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    let descriptor = ResourceDescriptor {
        id: ResourceId::new(1),
        revision: Revision::INITIAL,
        kind: ResourceKind::Font,
        byte_len: bytes.len() as u64,
    };
    let fonts = chart_export::FontResources::new(vec![
        chart_export::FontResource::new(descriptor, std::sync::Arc::from(bytes)).unwrap(),
    ])
    .unwrap();
    let request = LayoutRequest::new(
        Rect::new(0., 0., 600., 360.).unwrap(),
        Units::Points,
        descriptor,
    );
    layout(prepared, &request, &fonts)
        .unwrap()
        .scene()
        .items()
        .iter()
        .map(|i| i.primitive.clone())
        .collect()
}
#[test]
fn append_upsert_remove_retention_match_batch_and_failed_update_rolls_back() {
    let k = 9_007_199_254_740_993;
    let initial = [(k, 0., 1., "a"), (k + 1, 1., 2., "b")];
    let source = data(&initial);
    let p = author(source.clone());
    let mut chart = p.chart().unwrap();
    let old = chart.prepare().unwrap();
    let old_ink = primitives(old.clone());
    let rows = vec![(k, 0., 1., "a"), (k + 1, 1., 2., "b"), (k + 2, 2., 3., "c")];
    let tx = chart
        .transaction()
        .unwrap()
        .append(&source, data(&rows[2..]))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Applied(_)
    ));
    assert_eq!(
        primitives(chart.prepare().unwrap()),
        primitives(author(data(&rows)).chart().unwrap().prepare().unwrap())
    );
    let rows = vec![(k, 0., 1., "a"), (k + 1, 4., 5., "c"), (k + 2, 2., 3., "c")];
    let tx = chart
        .transaction()
        .unwrap()
        .upsert(&source, data(&rows[1..2]))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Applied(_)
    ));
    assert_eq!(
        primitives(chart.prepare().unwrap()),
        primitives(author(data(&rows)).chart().unwrap().prepare().unwrap())
    );
    let tx = chart
        .transaction()
        .unwrap()
        .remove(&source, [k])
        .retain_count(&source, Some(1))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(tx).unwrap(),
        CommitOutcome::Applied(_)
    ));
    let expected = vec![rows[2]];
    let actual = primitives(chart.prepare().unwrap());
    assert_eq!(
        actual,
        primitives(author(data(&expected)).chart().unwrap().prepare().unwrap())
    );
    let invalid = chart
        .transaction()
        .unwrap()
        .append(&source, data(&expected))
        .build()
        .unwrap();
    assert!(matches!(
        chart.apply_transaction(invalid).unwrap(),
        CommitOutcome::Rejected(_)
    ));
    assert_eq!(actual, primitives(chart.prepare().unwrap()));
    assert_eq!(old.panels().len(), 2);
    assert_eq!(old_ink, primitives(old));
}
