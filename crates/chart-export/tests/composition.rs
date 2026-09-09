//! WP-13 FIX-12/13: shared preparation, real font shaping and composed destination contracts.
use chart_core::{
    composition::*, grammar::*, inspection::*, layout::*, portable::Session, scene::*, services::*,
    theme::*, typography::*, *,
};
use chart_export::{portable::PortableChart, *};
use std::sync::Arc;
const CASES: &str = include_str!("../../../fixtures/composition/portable-cases.json");
const PROFILE: &str = include_str!("../../../fixtures/composition/profile.json");
const REGULAR: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../../../fixtures/composition/fonts/NotoSans-Bold.ttf");
const ARABIC: &[u8] =
    include_bytes!("../../../fixtures/composition/fonts/NotoSansArabic-Regular.ttf");
fn descriptor(id: u64, bytes: &[u8]) -> ResourceDescriptor {
    ResourceDescriptor {
        id: ResourceId::new(id),
        revision: Revision::new(if id == 9007199254747001 { 3 } else { 1 }),
        kind: ResourceKind::Font,
        byte_len: bytes.len() as u64,
    }
}
fn fonts() -> FontResources {
    FontResources::new(
        [
            (9007199254747001, REGULAR),
            (9007199254747002, BOLD),
            (9007199254747003, ARABIC),
        ]
        .into_iter()
        .map(|(id, b)| FontResource::new(descriptor(id, b), Arc::from(b)).unwrap())
        .collect(),
    )
    .unwrap()
}
fn cases() -> Vec<serde_json::Value> {
    serde_json::from_str(CASES).unwrap()
}
fn session() -> Session {
    let c = &cases()[0];
    Session::new(&c["chart"].to_string(), &c["data"].to_string()).unwrap()
}
fn portable(index: usize) -> PortableChart {
    let c = &cases()[index];
    PortableChart::new(
        &c["chart"].to_string(),
        &c["data"].to_string(),
        PROFILE,
        REGULAR.to_vec(),
    )
    .unwrap()
}
fn request() -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 180. / 25.4 * 72., 120. / 25.4 * 72.).unwrap(),
        Units::Points,
        descriptor(9007199254747001, REGULAR),
    )
}
fn prepare(s: &Session, d: &ChartDefinition) -> Arc<PreparedChart> {
    Arc::new(
        Compiler::new()
            .prepare(d, &s.source(), s.state(), CompileLimits::default())
            .unwrap(),
    )
}

#[test]
fn fix12_three_themes_reuse_stat_tables_and_preserve_numerical_geometry() {
    let s = session();
    let source = s.source();
    let mut compiler = Compiler::new();
    let mut previous: Option<Arc<PreparedChart>> = None;
    let mut scenes = vec![];
    for (index, case) in cases().iter().enumerate() {
        let mut definition: ChartDefinition =
            serde_json::from_value(case["chart"]["definition"].clone()).unwrap();
        definition.revision = Revision::new(13 + index as u64);
        let prepared = Arc::new(
            compiler
                .prepare(&definition, &source, s.state(), CompileLimits::default())
                .unwrap(),
        );
        if let Some(p) = &previous {
            for (a, b) in p.layers().iter().zip(prepared.layers()) {
                assert_eq!(a.marks(), b.marks());
                assert_eq!(a.marks().as_ptr(), b.marks().as_ptr());
                assert_eq!(a.domains(), b.domains());
                assert_eq!(a.color_legend(), b.color_legend());
                assert!(
                    Arc::ptr_eq(a.table(), b.table()),
                    "theme must reuse exact prepared table"
                );
            }
        }
        let laid = layout(prepared.clone(), &request(), &fonts()).unwrap();
        assert!(Arc::ptr_eq(laid.prepared(), &prepared));
        assert!(
            laid.panels()
                .iter()
                .all(|p| p.chart.status() == LayoutStatus::Ready)
        );
        let inset = &laid.insets()[0];
        let parent = laid
            .panels()
            .iter()
            .find(|p| Some(&p.key) == inset.panel.as_ref())
            .unwrap();
        for layer in inset.chart.prepared().layers() {
            let parent = parent
                .chart
                .prepared()
                .layers()
                .iter()
                .find(|l| l.id() == layer.id())
                .unwrap();
            assert!(Arc::ptr_eq(layer.table(), parent.table()));
            assert_eq!(layer.marks(), parent.marks());
        }
        scenes.push(serde_json::to_value(laid.scene().items()).unwrap());
        previous = Some(prepared);
    }
    assert_ne!(scenes[0], scenes[1]);
    assert_ne!(scenes[0], scenes[2]);
    let mut a = portable(0);
    let mut b = portable(1);
    let mut c = portable(2);
    assert_eq!(a.semantics().unwrap(), b.semantics().unwrap());
    assert_eq!(a.semantics().unwrap(), c.semantics().unwrap());
}
#[test]
fn fix13_exact_faces_rich_rotation_inset_vectors_and_real_raster_dimensions() {
    for index in 0..3 {
        let p = portable(index);
        let f = p.capture().unwrap();
        assert_eq!(f.layout().panels().len(), 2);
        assert_eq!(f.layout().insets().len(), 1);
        assert_eq!(f.metadata().fonts.len(), 3);
        let items = f.scene().items();
        assert!(
            items
                .iter()
                .any(|i| matches!(i.primitive, Primitive::GradientRectangle { .. }))
        );
        assert!(
            items
                .iter()
                .any(|i| matches!(i.primitive, Primitive::DashedPath { .. }))
        );
        assert!(
            items
                .iter()
                .any(|i| matches!(i.primitive, Primitive::Symbol { .. }))
        );
        let rich: Vec<_> = items
            .iter()
            .filter_map(|i| {
                if let Primitive::GlyphRun { run, rotation, .. } = &i.primitive {
                    Some((run, *rotation))
                } else {
                    None
                }
            })
            .collect();
        assert!(rich.iter().any(|(r, angle)| *angle == -35. && r.tabular));
        assert!(
            rich.iter()
                .any(|(r, _)| r.font.id == ResourceId::new(9007199254747002))
        );
        let arabic = rich.iter().find(|(r, _)| r.language == "ar").unwrap().0;
        assert_eq!(arabic.text, "مرحبا بالعالم");
        assert_eq!(arabic.direction, TextDirection::RightToLeft);
        assert!(arabic.used_fallback);
        assert_eq!(arabic.font.id, ResourceId::new(9007199254747003));
        assert!(
            f.layout()
                .diagnostics()
                .iter()
                .all(|d| d.code == DiagnosticCode::MissingResource
                    && d.severity == Severity::Warning)
        );
        let svg = f.export(Format::Svg).unwrap();
        assert_eq!(
            svg.capabilities.text,
            TextRepresentation::MixedPositionedOutlines
        );
        let svg = String::from_utf8(svg.bytes).unwrap();
        assert!(svg.contains("<linearGradient"));
        assert!(svg.contains("stroke-dasharray"));
        assert!(svg.contains("<desc>مرحبا بالعالم</desc>"));
        assert!(!svg.contains("<image"));
        assert!(f.export(Format::Pdf).unwrap().bytes.starts_with(b"%PDF-"));
        if index == 0 {
            for (dpi, dimensions) in [(300, (2126, 1417)), (600, (4252, 2835))] {
                let mut profile: serde_json::Value = serde_json::from_str(PROFILE).unwrap();
                profile["dpi"] = dpi.into();
                let c = &cases()[0];
                let p = PortableChart::new(
                    &c["chart"].to_string(),
                    &c["data"].to_string(),
                    &profile.to_string(),
                    REGULAR.to_vec(),
                )
                .unwrap();
                let png = p.export("png").unwrap();
                let reader = png::Decoder::new(png.as_slice()).read_info().unwrap();
                assert_eq!((reader.info().width, reader.info().height), dimensions);
            }
        }
    }
}
#[test]
fn rich_shaping_has_independent_advances_real_weights_and_explicit_fallback() {
    let fonts = fonts();
    let default = descriptor(9007199254747001, REGULAR);
    let shape = |run: &RichRun| {
        fonts.shape(ShapeRequest {
            run,
            default_font: &default,
            font_size: 10.,
            units: Units::Points,
            limits: Limits::default(),
        })
    };
    let mut digits = RichRun::new("012345");
    digits.tabular = true;
    let result = shape(&digits).unwrap();
    assert!(
        (result.metrics.width() - 34.32).abs() < 1e-12,
        "Noto hmtx: 6 * 572/1000 * 10"
    );
    assert!(
        result
            .glyphs
            .iter()
            .all(|g| (g.advance.x() - 5.72).abs() < 1e-12)
    );
    let mut bold = RichRun::new("Title");
    bold.font = Some(descriptor(9007199254747002, BOLD));
    bold.weight = 700;
    assert!(shape(&bold).is_ok());
    bold.weight = 400;
    assert_eq!(
        shape(&bold).unwrap_err().code,
        DiagnosticCode::MissingResource
    );
    let mut arabic = RichRun::new("سلام");
    arabic.language = "ar".into();
    arabic.direction = TextDirection::RightToLeft;
    assert_eq!(
        shape(&arabic).unwrap_err().code,
        DiagnosticCode::MissingResource
    );
    arabic.fallback.push(descriptor(9007199254747003, ARABIC));
    let run = shape(&arabic).unwrap();
    assert!(run.used_fallback);
    let face = ttf_parser::Face::parse(ARABIC, 0).unwrap();
    let nominal: Vec<_> = "سلام"
        .chars()
        .rev()
        .map(|c| face.glyph_index(c).unwrap().0)
        .collect();
    assert_ne!(
        run.glyphs.iter().map(|g| g.id).collect::<Vec<_>>(),
        nominal,
        "Arabic uses contextual glyph forms, not nominal cmap glyphs"
    );
    let mut covered = vec![false; "سلام".len()];
    for g in &run.glyphs {
        covered[g.start..g.end].fill(true);
    }
    assert!(covered.into_iter().all(|v| v));
    assert!(run.glyphs.windows(2).all(|w| w[0].start >= w[1].start));
    arabic.text = "🦄".into();
    assert_eq!(
        shape(&arabic).unwrap_err().code,
        DiagnosticCode::MissingResource
    );
    digits.language = "en_US".into();
    assert_eq!(shape(&digits).unwrap_err().code, DiagnosticCode::Validation);
    digits.language = "en-US".into();
    digits.fallback.push(ResourceDescriptor {
        revision: Revision::new(99),
        ..descriptor(9007199254747003, ARABIC)
    });
    digits.text = "سلام".into();
    assert_eq!(
        shape(&digits).unwrap_err().context.resource,
        Some(ResourceId::new(9007199254747003))
    );
}
fn annotation(id: &str, anchor: Anchor, collision: Collision, priority: i32) -> Annotation {
    Annotation {
        id: id.into(),
        anchor,
        text: RichText::plain(id),
        offset: [0., 0.],
        priority,
        collision,
        callout: None,
        connector_origin: chart_core::composition::ConnectorOrigin::Label,
        overflow: true,
    }
}
fn rich_origin(c: &LaidOutChart, text: &str) -> Point {
    c.scene()
        .items()
        .iter()
        .find_map(|i| {
            if let Primitive::GlyphRun { origin, run, .. } = &i.primitive {
                (run.text == text).then_some(*origin)
            } else {
                None
            }
        })
        .unwrap()
}
#[test]
fn furniture_coordinate_spaces_resize_and_bounded_collision_policies() {
    let s = session();
    let mut d = s.definition().clone();
    d.figure = Some(FigureComposition {
        annotations: vec![
            annotation(
                "relative",
                Anchor::Figure { x: 0.5, y: 0.5 },
                Collision::Keep,
                0,
            ),
            annotation(
                "absolute",
                Anchor::Output { x: 20., y: 20. },
                Collision::Keep,
                0,
            ),
            annotation(
                "winner",
                Anchor::Output { x: 50., y: 50. },
                Collision::Keep,
                10,
            ),
            annotation(
                "hidden",
                Anchor::Output { x: 50., y: 50. },
                Collision::Hide,
                1,
            ),
        ],
        ..Default::default()
    });
    let prepared = prepare(&s, &d);
    let mut r = request();
    let a = layout(prepared.clone(), &r, &fonts()).unwrap();
    r.bounds = Rect::new(0., 0., r.bounds.width() + 100., r.bounds.height() + 80.).unwrap();
    let b = layout(prepared.clone(), &r, &fonts()).unwrap();
    let delta = |text| {
        let a = rich_origin(&a, text);
        let b = rich_origin(&b, text);
        (b.x() - a.x(), b.y() - a.y())
    };
    assert_eq!(delta("relative"), (50., 40.));
    assert_eq!(delta("absolute"), (0., 0.));
    assert!(
        a.scene()
            .items()
            .iter()
            .all(|i| !matches!(&i.primitive,Primitive::GlyphRun{run,..} if run.text=="hidden"))
    );
    assert!(
        a.diagnostics()
            .iter()
            .any(|d| d.code == DiagnosticCode::LayoutPressure)
    );
    r.bounds = Rect::new(0., 0., 20., 20.).unwrap();
    let tiny = layout(prepared, &r, &fonts()).unwrap();
    assert_eq!(tiny.status(), LayoutStatus::NoSpace);
}
#[test]
fn theme_cascade_mapped_colors_symbols_and_full_domain_output_contract() {
    let s = session();
    let mut d = s.definition().clone();
    d.figure = None;
    let theme = d.theme.as_mut().unwrap();
    theme.plot.mark = Some(rgb(20, 30, 40).into());
    theme.layers.get_mut(&LayerId::new(3)).unwrap().mark = Some(rgb(50, 60, 70).into());
    let mut r = request();
    r.host_theme.mark = Some(rgb(1, 2, 3).into());
    r.interaction_theme.insert(
        LayerId::new(3),
        ThemePatch {
            mark: Some(rgb(80, 90, 100).into()),
            ..Default::default()
        },
    );
    r.output_theme.mark = Some(rgb(110, 120, 130).into());
    let laid = Arc::new(layout(prepare(&s, &d), &r, &fonts()).unwrap());
    assert!(laid.scene().items().iter().filter(|i|i.layer==Some(LayerId::new(3))).all(|i|matches!(&i.primitive,Primitive::Path{stroke,..} if stroke.color==rgb(110,120,130))));
    let center = laid
        .scene()
        .items()
        .iter()
        .find_map(|i| {
            if let Primitive::Symbol { center, fill, .. } = &i.primitive {
                assert_ne!(*fill, rgb(110, 120, 130));
                Some(*center)
            } else {
                None
            }
        })
        .unwrap();
    let mut inspector = Inspector::new(laid.clone(), 4., 100).unwrap();
    inspector
        .dispatch(
            laid.scene().stamp(),
            InspectionAction::Hover(Some(center)),
            InputOrigin::Pointer,
        )
        .unwrap();
    assert_eq!(inspector.hits()[0].position, center);
    d.axes[0].viewport = Some(chart_core::scales::Bounds::new(0.5, 1.5).unwrap());
    let mut profile =
        PublicationProfile::new(PageSize::millimeters(180., 120.).unwrap(), r.font).unwrap();
    profile.view = ViewMode::FullDomain;
    let full = FigureSnapshot::capture(&d, s.source(), s.state(), fonts(), profile).unwrap();
    assert!(
        full.layout()
            .panels()
            .iter()
            .all(|p| p.chart.axes()[&ScaleId::new(0)].spec.viewport.is_none())
    );
}
#[test]
fn invalid_resources_themes_furniture_and_dashes_fail_without_paint() {
    let s = session();
    let mut d = s.definition().clone();
    d.theme.as_mut().unwrap().version = 99;
    assert_eq!(
        Compiler::new()
            .prepare(&d, &s.source(), s.state(), CompileLimits::default())
            .unwrap_err()
            .code,
        DiagnosticCode::UnsupportedCapability
    );
    d = s.definition().clone();
    d.figure.as_mut().unwrap().insets[0].rectangle[2] = 2.;
    assert_eq!(
        Compiler::new()
            .prepare(&d, &s.source(), s.state(), CompileLimits::default())
            .unwrap_err()
            .code,
        DiagnosticCode::Validation
    );
    let commands = vec![
        PathCommand::MoveTo(Point::new(0., 0.).unwrap()),
        PathCommand::LineTo(Point::new(10., 0.).unwrap()),
        PathCommand::LineTo(Point::new(10., 10.).unwrap()),
    ];
    let dashed = dash_polyline(&commands, &[6., 4.], 100).unwrap();
    assert_eq!(
        dashed,
        vec![
            PathCommand::MoveTo(Point::new(0., 0.).unwrap()),
            PathCommand::LineTo(Point::new(6., 0.).unwrap()),
            PathCommand::MoveTo(Point::new(10., 0.).unwrap()),
            PathCommand::LineTo(Point::new(10., 6.).unwrap())
        ]
    );
    assert_eq!(
        dash_polyline(&commands, &[0.000001, 0.000001], 10)
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let f = NumberFormat {
        notation: NumberNotation::Fixed,
        precision: 2,
        locale: NumberLocale::FrFr,
        grouping: true,
        prefix: String::new(),
        suffix: " kg".into(),
    };
    assert_eq!(f.format(-1234.5).unwrap(), "−1\u{202f}234,50 kg");
}
