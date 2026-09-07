//! WP-12: independently expected populations, shared/free training and bounded pane layout.
use chart_core::{
    data::*, grammar::*, inspection::*, layout::*, provenance::*, scales::*, scene::*, services::*,
    state::*, transaction::*, *,
};
use std::{cell::Cell, sync::Arc};
const DATA: DatasetId = DatasetId::new(1);
const NOTES: DatasetId = DatasetId::new(2);
const X: FieldId = FieldId::new(1);
const Y: FieldId = FieldId::new(2);
const F: FieldId = FieldId::new(3);
const G: FieldId = FieldId::new(4);
const FONT: ResourceDescriptor = ResourceDescriptor {
    id: ResourceId::new(1),
    revision: Revision::INITIAL,
    kind: ResourceKind::Font,
    byte_len: 10,
};
struct Metrics {
    width: f64,
    calls: Cell<usize>,
}
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        self.calls.set(self.calls.get() + 1);
        TextMetrics::new(self.width * r.text.chars().count() as f64, 9., 3.)
    }
}
fn key(s: &str) -> PanelKey {
    PanelKey {
        values: vec![GroupValue::Text(s.into())],
    }
}
fn fixture() -> DataStore {
    let schema = Arc::new(
        Schema::new(
            SchemaVersion::new(1),
            [
                (X, "x", FieldKind::Float64),
                (Y, "y", FieldKind::Float64),
                (F, "facet", FieldKind::Utf8),
                (G, "group", FieldKind::Int64),
            ]
            .into_iter()
            .map(|(id, name, kind)| Field {
                id,
                name: name.into(),
                kind,
                nullable: false,
                unit: None,
                label: None,
            })
            .collect(),
        )
        .unwrap(),
    );
    let batch = NormalizedBatch::new(
        schema,
        (0..5).map(|i| RowKey::new((1 << 53) + i + 1)).collect(),
        vec![
            Column::new(
                ColumnValues::Float64(vec![0., 1., 2., 0., 1.]),
                vec![true; 5],
                None,
            ),
            Column::new(
                ColumnValues::Float64(vec![1., 3., 9., 100., 120.]),
                vec![true; 5],
                None,
            ),
            Column::new(
                ColumnValues::Utf8(vec![
                    "A".into(),
                    "A".into(),
                    "A".into(),
                    "B".into(),
                    "B".into(),
                ]),
                vec![true; 5],
                None,
            ),
            Column::new(
                ColumnValues::Int64(vec![1, 1, 2, 1, 2]),
                vec![true; 5],
                None,
            ),
        ],
        DataLimits::default(),
    )
    .unwrap();
    let schema = Arc::new(
        Schema::new(
            SchemaVersion::new(1),
            vec![Field {
                id: Y,
                name: "threshold".into(),
                kind: FieldKind::Float64,
                nullable: false,
                unit: None,
                label: None,
            }],
        )
        .unwrap(),
    );
    let notes = NormalizedBatch::new(
        schema,
        vec![RowKey::new(99)],
        vec![Column::new(
            ColumnValues::Float64(vec![50.]),
            vec![true],
            None,
        )],
        DataLimits::default(),
    )
    .unwrap();
    DataStore::new(
        SourceEpoch::new(1),
        vec![(DATA, batch), (NOTES, notes)],
        DataLimits::default(),
    )
    .unwrap()
}
fn specification() -> FacetSpec {
    FacetSpec {
        fields: vec![F],
        order: vec![key("B"), key("A")],
        layout: FacetLayout::Wrap { columns: 2 },
        empty: EmptyPanels::Keep,
        scales: FacetScales::default(),
        gap: 12.,
        collect_guides: true,
    }
}
fn definition() -> ChartDefinition {
    let mut d = ChartDefinition::new(Revision::new(1)).layer(Layer::new(
        LayerId::new(1),
        DATA,
        Geom::Point,
        SourceAes::new().x(X).y(Y).group(G),
    ));
    d.facets = Some(specification());
    d
}
fn prepare(d: &ChartDefinition, s: &DataStore) -> ChartResult<PreparedChart> {
    Compiler::new().prepare(
        d,
        &s.snapshot(),
        &ChartState::default(),
        CompileLimits::default(),
    )
}
fn request(w: f64, h: f64) -> LayoutRequest {
    LayoutRequest::new(Rect::new(0., 0., w, h).unwrap(), Units::Points, FONT)
}
fn draw(p: PreparedChart, w: f64, h: f64, width: f64) -> LaidOutChart {
    layout(
        Arc::new(p),
        &request(w, h),
        &Metrics {
            width,
            calls: Cell::new(0),
        },
    )
    .unwrap()
}
fn domain(p: &LaidOutPanel, id: u64) -> Bounds {
    let ResolvedScale::Linear(s) = &p.chart.axes()[&ScaleId::new(id)].scale else {
        panic!("linear")
    };
    s.domain()
}
fn annotation() -> Layer {
    Layer::new(
        LayerId::new(2),
        NOTES,
        Geom::Rule,
        SourceAes::new()
            .x(Numeric::Literal(0.))
            .x2(Numeric::Literal(2.))
            .y(Y)
            .y2(Y),
    )
    .independent()
}

#[test]
fn fix06_explicit_annotation_policy_shared_domain_and_exact_panel_targets() {
    let store = fixture();
    let mut d = definition();
    d.layers.push(annotation());
    assert_eq!(
        prepare(&d, &store).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
    d.layers[1].facet = FacetTarget::Broadcast;
    let p = prepare(&d, &store).unwrap();
    assert_eq!(
        p.panels().iter().map(|p| p.key.clone()).collect::<Vec<_>>(),
        vec![key("B"), key("A")]
    );
    assert_eq!(
        p.panels()
            .iter()
            .map(|p| p.chart.layers()[0].table().rows().len())
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert!(
        p.panels()
            .iter()
            .all(|p| p.chart.layers()[1].marks().len() == 1)
    );
    let drawn = Arc::new(draw(p, 700., 350., 6.));
    assert_eq!(
        domain(&drawn.panels()[0], 1),
        Bounds::new(1., 120.).unwrap()
    );
    assert_eq!(domain(&drawn.panels()[0], 1), domain(&drawn.panels()[1], 1));
    assert_eq!(
        drawn.panels()[0].chart.plot().unwrap().origin().y(),
        drawn.panels()[1].chart.plot().unwrap().origin().y()
    );
    assert_eq!(
        drawn.panels()[0].chart.plot().unwrap().width(),
        drawn.panels()[1].chart.plot().unwrap().width()
    );
    assert_eq!(drawn.item_panels().len(), drawn.scene().items().len());
    let mut inspector = Inspector::new(drawn.clone(), 5., 8).unwrap();
    inspector
        .dispatch(
            drawn.scene().stamp(),
            InspectionAction::StepFocus { forward: true },
            InputOrigin::Keyboard,
        )
        .unwrap();
    assert_eq!(inspector.hits()[0].panel, Some(key("B")));
    let Target::Source(source) = inspector.hits()[0].target else {
        panic!("source")
    };
    assert_eq!(source.key, RowKey::new((1 << 53) + 4));
    let snapshot = store.snapshot();
    assert!(matches!(
        inspector.hits()[0]
            .target
            .resolve(snapshot.get().unwrap())
            .unwrap(),
        ResolvedTarget::Source(_)
    ));
    d.layers[1].facet = FacetTarget::Panels(vec![key("A")]);
    let p = prepare(&d, &store).unwrap();
    assert_eq!(p.panels()[0].chart.layers().len(), 1);
    assert_eq!(p.panels()[1].chart.layers().len(), 2);
}

#[test]
fn free_scales_intentionally_differ_and_resize_font_changes_keep_aligned_panes() {
    let s = fixture();
    let mut d = definition();
    d.facets.as_mut().unwrap().scales.free_y = true;
    for (w, h, font) in [(700., 350., 6.), (420., 210., 9.), (900., 500., 12.)] {
        let c = draw(prepare(&d, &s).unwrap(), w, h, font);
        assert_eq!(c.status(), LayoutStatus::Ready);
        assert!(c.passes() <= MAX_LAYOUT_PASSES);
        assert_eq!(domain(&c.panels()[0], 1), Bounds::new(100., 120.).unwrap());
        assert_eq!(domain(&c.panels()[1], 1), Bounds::new(1., 9.).unwrap());
        let a = c.panels()[0].chart.plot().unwrap();
        let b = c.panels()[1].chart.plot().unwrap();
        assert_eq!(a.width(), b.width());
        assert_eq!(a.height(), b.height());
        assert_eq!(a.origin().y(), b.origin().y());
    }
    let tiny = draw(prepare(&d, &s).unwrap(), 35., 20., 6.);
    assert_eq!(tiny.status(), LayoutStatus::NoSpace);
    assert!(
        tiny.diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::LayoutPressure)
    );
}

#[test]
fn group_facet_chart_statistics_have_independent_expected_populations_and_cache_keys() {
    let s = fixture();
    let mut d = definition();
    let mut summary = SummarySpec::new(Y);
    summary.grouping = Grouping::Field(G);
    let node = TransformDefinition::new(TransformId::new(1), DATA, Statistic::summary(summary));
    d.transforms.push(node);
    d.layers = vec![Layer::statistical(
        LayerId::new(1),
        TransformId::new(1),
        Statistic::identity(),
        Geom::Point,
        StatAes::new(StatField::Group, StatField::Mean),
    )];
    let mut c = Compiler::new();
    let snapshot = s.snapshot();
    let p = c
        .prepare(
            &d,
            &snapshot,
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    let means = |p: &PreparedChart| {
        p.panels()
            .iter()
            .map(|p| {
                let PreparedRows::Statistical(rows) = p.chart.layers()[0].table().rows() else {
                    panic!("stat")
                };
                rows.iter()
                    .map(|r| r.value(&StatField::Mean).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(means(&p), vec![vec![100., 120.], vec![2., 9.]]);
    let cached = c
        .prepare(
            &d,
            &snapshot,
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(cached.metrics().evaluated_transforms, 0);
    assert_eq!(cached.metrics().reused_transforms, 2);
    d.transforms[0].scope = StatScope::Facet;
    let facet = c
        .prepare(
            &d,
            &snapshot,
            &ChartState::default(),
            CompileLimits::default(),
        )
        .unwrap();
    assert_eq!(means(&facet), vec![vec![110.], vec![13. / 3.]]);
    assert_eq!(
        facet.panels()[0].chart.layers()[0].table().operations()[0].panel,
        Some(key("B"))
    );
    d.transforms[0].scope = StatScope::Chart;
    assert!(prepare(&d, &s).is_err()); // A chart aggregate requires explicit presentation targets.
    d.layers[0].facet = FacetTarget::Broadcast;
    let chart = prepare(&d, &s).unwrap();
    assert_eq!(means(&chart), vec![vec![233. / 5.], vec![233. / 5.]]);
    assert_eq!(
        chart.panels()[0].chart.layers()[0].table().operations()[0].panel,
        None
    );
    assert_eq!(
        chart.panels()[0].chart.layers()[0].marks()[0].targets,
        chart.panels()[1].chart.layers()[0].marks()[0].targets
    );
}

#[test]
fn explicit_empty_panels_grid_coordinates_and_reordering_preserve_keys() {
    let s = fixture();
    let mut d = definition();
    d.facets.as_mut().unwrap().order.push(key("C"));
    let p = prepare(&d, &s).unwrap();
    assert_eq!(p.panels().len(), 3);
    assert!(p.panels()[2].chart.layers()[0].marks().is_empty());
    assert_eq!((p.panels()[2].row, p.panels()[2].column), (1, 0));
    d.facets.as_mut().unwrap().empty = EmptyPanels::Drop;
    assert_eq!(prepare(&d, &s).unwrap().panels().len(), 2);
    let spec = d.facets.as_mut().unwrap();
    spec.fields = vec![F, G];
    spec.layout = FacetLayout::Grid;
    spec.empty = EmptyPanels::Keep;
    spec.order = [("B", 2), ("B", 1), ("A", 2), ("A", 1)]
        .into_iter()
        .map(|(f, g)| PanelKey {
            values: vec![GroupValue::Text(f.into()), GroupValue::Int(g)],
        })
        .collect();
    let p = prepare(&d, &s).unwrap();
    assert_eq!(
        p.panels()
            .iter()
            .map(|p| (p.row, p.column))
            .collect::<Vec<_>>(),
        vec![(0, 0), (0, 1), (1, 0), (1, 1)]
    );
    let c = draw(p, 700., 500., 6.);
    assert_eq!(c.panels().len(), 4);
    assert_eq!(c.status(), LayoutStatus::Ready);
    let a = c.panels()[0].chart.plot().unwrap();
    let b = c.panels()[2].chart.plot().unwrap();
    assert_eq!(a.origin().x(), b.origin().x());
    assert_eq!(a.width(), b.width());
    d.facets.as_mut().unwrap().order.pop();
    assert!(prepare(&d, &s).is_err());
}

#[test]
fn color_guides_collect_only_exact_compatible_id_domain_and_palette() {
    let s = fixture();
    let mut d = definition();
    let blue = Color {
        red: 0,
        green: 80,
        blue: 180,
        alpha: 255,
    };
    let red = Color {
        red: 200,
        green: 20,
        blue: 40,
        alpha: 255,
    };
    d.layers[0].color = Some(ColorEncoding {
        title: Some("Groups".into()),
        id: ScaleId::new(7),
        input: ColorInput::Category(F),
        scale: ColorScale::Discrete {
            domain: Some(vec!["A".into(), "B".into()]),
            palette: vec![blue, red],
            missing: blue,
        },
    });
    let count = |c: &LaidOutChart| {
        c.scene()
            .items()
            .iter()
            .filter(|i| matches!(&i.primitive,Primitive::Text{text,..} if text=="Groups"))
            .count()
    };
    assert_eq!(count(&draw(prepare(&d, &s).unwrap(), 800., 350., 6.)), 1);
    d.facets.as_mut().unwrap().collect_guides = false;
    assert_eq!(count(&draw(prepare(&d, &s).unwrap(), 800., 350., 6.)), 2);
    d.facets.as_mut().unwrap().collect_guides = true;
    d.layers[0].facet = FacetTarget::Panels(vec![key("A")]);
    let mut second = d.layers[0].clone();
    second.id = LayerId::new(2);
    second.facet = FacetTarget::Panels(vec![key("B")]);
    if let ColorScale::Discrete { palette, .. } = &mut second.color.as_mut().unwrap().scale {
        palette.reverse();
    }
    d.layers.push(second);
    assert_eq!(count(&draw(prepare(&d, &s).unwrap(), 800., 350., 6.)), 2);
}

#[test]
fn figure_budgets_invalid_catalog_and_tiny_dense_labels_fail_or_fallback_explicitly() {
    let s = fixture();
    let d = definition();
    let mut limits = CompileLimits {
        max_prepared_rows: 5,
        ..Default::default()
    };
    assert!(
        Compiler::new()
            .prepare(&d, &s.snapshot(), &ChartState::default(), limits)
            .is_ok()
    );
    limits.max_prepared_rows = 4;
    assert_eq!(
        Compiler::new()
            .prepare(&d, &s.snapshot(), &ChartState::default(), limits)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let mut invalid = d.clone();
    invalid.facets.as_mut().unwrap().order = vec![key("A")];
    assert!(prepare(&invalid, &s).is_err());
    invalid.facets.as_mut().unwrap().order = vec![key("A"), key("A")];
    assert!(prepare(&invalid, &s).is_err());
    let c = draw(prepare(&d, &s).unwrap(), 400., 150., 40.);
    assert!(c.passes() <= 4);
    assert!(
        c.diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::LayoutPressure)
    );
    let mut r = request(400., 150.);
    r.limits.max_items = 2;
    let metrics = Metrics {
        width: 6.,
        calls: Cell::new(0),
    };
    assert_eq!(
        layout(Arc::new(prepare(&d, &s).unwrap()), &r, &metrics)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
}

#[test]
fn shared_log_scale_uses_positive_population_from_all_panels() {
    let s = fixture();
    let mut d = definition();
    let mut axis = AxisSpec::new(ScaleId::new(1), AxisSide::Left);
    axis.scale = AxisScale::Nonlinear {
        transform: ScaleTransform::Log { base: 10. },
        domain: ContinuousDomain::default(),
    };
    d.axes = vec![AxisSpec::new(ScaleId::new(0), AxisSide::Bottom), axis];
    let c = draw(prepare(&d, &s).unwrap(), 700., 350., 6.);
    for p in c.panels() {
        let ResolvedScale::Nonlinear(s) = &p.chart.axes()[&ScaleId::new(1)].scale else {
            panic!("log")
        };
        assert!((s.domain().start() - 1.).abs() < 1e-12);
        assert!((s.domain().end() - 120.).abs() < 1e-12);
    }
}

#[test]
fn source_filters_precede_catalog_validation_and_empty_panel_drop() {
    let s = fixture();
    let mut d = definition();
    d.layers[0].filters.push(SourceFilter {
        value: Y.into(),
        minimum: None,
        maximum: Some(10.),
    });
    d.facets.as_mut().unwrap().empty = EmptyPanels::Drop;
    let p = prepare(&d, &s).unwrap();
    assert_eq!(p.panels().len(), 1);
    assert_eq!(p.panels()[0].key, key("A"));
    // A fully filtered key need not appear in the authored catalog.
    d.facets.as_mut().unwrap().order = vec![key("A")];
    assert!(prepare(&d, &s).is_ok());
    // The same predicate also composes through a named identity dependency.
    let mut node = TransformDefinition::new(TransformId::new(1), DATA, Statistic::identity());
    node.filters = std::mem::take(&mut d.layers[0].filters);
    d.transforms.push(node);
    d.layers[0].data = TransformId::new(1).into();
    assert_eq!(
        prepare(&d, &s).unwrap().panels()[0].chart.layers()[0]
            .table()
            .rows()
            .len(),
        3
    );
}

#[test]
fn declared_figure_overflow_and_default_plot_clip_are_preserved_in_facets() {
    let s = fixture();
    let mut d = definition();
    let mut note = annotation();
    note.facet = FacetTarget::Broadcast;
    note.clip = ClipPolicy::Figure;
    d.layers.push(note);
    let c = draw(prepare(&d, &s).unwrap(), 700., 350., 6.);
    for (item, panel) in c.scene().items().iter().zip(c.item_panels()) {
        if item.layer == Some(LayerId::new(2)) {
            assert_eq!(item.clip, Some(c.scene().bounds()));
        }
        if item.layer == Some(LayerId::new(1)) {
            let p = c
                .panels()
                .iter()
                .find(|p| Some(&p.key) == panel.as_ref())
                .unwrap();
            assert_eq!(item.clip, p.chart.plot());
        }
    }
}
