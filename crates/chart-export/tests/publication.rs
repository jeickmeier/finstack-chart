//! WP-08 FIX-13/14 subsets: actual headless bytes, shared geometry and immutable resources.
#[path = "../../../fixtures/publication/support.rs"]
mod fixture;
use chart_core::{grammar::*, scene::*, services::*, state::*, transaction::*, *};
use chart_export::*;
use fixture::*;
use std::sync::Arc;
fn capture(p: PublicationProfile) -> FigureSnapshot {
    FigureSnapshot::capture(
        &definition(),
        store().snapshot(),
        &ChartState::default(),
        fonts(),
        p,
    )
    .unwrap()
}
fn png_data(bytes: &[u8]) -> (png::OutputInfo, Vec<u8>, Option<png::PixelDimensions>) {
    let mut reader = png::Decoder::new(bytes).read_info().unwrap();
    let density = reader.info().pixel_dims;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).unwrap();
    (info, buffer, density)
}
#[test]
fn fix13_physical_dimensions_density_font_hash_and_vector_modes() {
    let mut p = profile();
    assert!((p.page.width() - 510.23622047244095).abs() < 1e-12);
    assert!((p.page.height() - 340.15748031496065).abs() < 1e-12);
    for (dpi, expected, ppm) in [(300, (2126, 1417), 11811), (600, (4252, 2835), 23622)] {
        p.dpi = dpi;
        assert_eq!(p.raster_dimensions().unwrap(), expected);
        let s = capture(p.clone());
        let artifact = s.export(Format::Png).unwrap();
        let (info, _, density) = png_data(&artifact.bytes);
        assert_eq!((info.width, info.height), expected);
        assert_eq!(density.unwrap().xppu, ppm);
        assert_eq!(
            s.metadata().fonts[0].sha256,
            "2ec33f84606cbaa0a1a944488e14f97faf2f6a25ecdd8354f5358f06da13c7d9"
        );
    }
    for text in [TextMode::Preserve, TextMode::Outline] {
        p.text = text;
        let s = capture(p.clone());
        let svg = String::from_utf8(s.export(Format::Svg).unwrap().bytes).unwrap();
        assert!(svg.contains("<path"));
        assert!(!svg.contains("<image"));
        assert_eq!(svg.contains("<text"), text == TextMode::Preserve);
        assert_eq!(
            svg.contains("data:font/ttf;base64,"),
            text == TextMode::Preserve
        );
        assert!(s.export(Format::Pdf).unwrap().bytes.starts_with(b"%PDF-"));
    }
}
#[test]
fn publication_metrics_use_supplied_face_and_real_kerning() {
    let f = font();
    let fonts = fonts();
    let measure = |text| {
        fonts
            .measure(TextRequest {
                text,
                font: &f.descriptor(),
                font_size: 10.,
                units: Units::Points,
            })
            .unwrap()
    };
    // Independent Noto Sans fixture hmtx: six tabular digits, 572 units each, UPEM=1000.
    assert!((measure("012345").width() - 34.32).abs() < 0.00001);
    assert!(measure("AV").width() < measure("A").width() + measure("V").width());
    assert!(measure("café Ω").height() > 0.);
}
#[test]
fn visible_and_full_domain_keep_bins_members_visibility_and_declared_state() {
    let d = ChartDefinition::new(Revision::new(1)).layer(Layer::histogram(
        LayerId::new(1),
        DATA,
        BinSpec::new(X, vec![0., 1., 2.]),
    ));
    let data = store().snapshot();
    let mut state = ChartState::default();
    state
        .apply(
            &d,
            ChartAction::SetViewport(Viewport {
                x: Some((0.5, 1.5)),
                y: None,
            }),
        )
        .unwrap();
    let visible = FigureSnapshot::capture(&d, data.clone(), &state, fonts(), profile()).unwrap();
    let mut p = profile();
    p.view = ViewMode::FullDomain;
    p.layout.axes[0].viewport = Some(chart_core::scales::Bounds::new(0.75, 1.25).unwrap());
    let full = FigureSnapshot::capture(&d, data.clone(), &state, fonts(), p.clone()).unwrap();
    let PreparedRows::Binned(bins) = visible.layout().prepared().layers()[0].table().rows() else {
        panic!("bins")
    };
    assert_eq!(bins.iter().map(|b| b.count).collect::<Vec<_>>(), vec![2, 3]);
    assert_eq!(
        visible.layout().prepared().layers()[0].table().rows(),
        full.layout().prepared().layers()[0].table().rows()
    );
    assert_ne!(
        visible.layout().scene().items(),
        full.layout().scene().items()
    );
    for format in [Format::Svg, Format::Pdf, Format::Png] {
        assert!(!full.export(format).unwrap().bytes.is_empty());
    }
    assert_eq!(full.metadata().captured_state, state);
    assert_eq!(
        full.metadata().profile.layout.axes[0].viewport,
        p.layout.axes[0].viewport
    );
    assert_eq!(
        full.layout().prepared().state().viewport(),
        Viewport::default()
    );
    assert!(
        full.layout()
            .axes()
            .values()
            .all(|a| a.spec.viewport.is_none())
    );
    state
        .apply(
            &d,
            ChartAction::SetLayerVisible {
                layer: LayerId::new(1),
                visible: false,
            },
        )
        .unwrap();
    let hidden = FigureSnapshot::capture(&d, data, &state, fonts(), p).unwrap();
    assert!(
        !hidden
            .layout()
            .prepared()
            .state()
            .is_visible(LayerId::new(1))
    );
}
#[test]
fn fix14_capture_survives_concurrent_data_and_annotation_edits_then_releases() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<FigureSnapshot>();
    let mut store = store();
    let mut p = profile();
    let mut d = definition();
    let bytes: Arc<[u8]> = Arc::from(FONT_BYTES);
    let weak_bytes = Arc::downgrade(&bytes);
    let f = FontResource::new(font().descriptor(), bytes.clone()).unwrap();
    drop(bytes);
    let snapshot = FigureSnapshot::capture(
        &d,
        store.snapshot(),
        &ChartState::default(),
        FontResources::new(vec![f]).unwrap(),
        p.clone(),
    )
    .unwrap();
    let weak_layout = Arc::downgrade(snapshot.layout());
    let before = snapshot.export(Format::Svg).unwrap().bytes;
    let worker = snapshot.clone();
    let thread = std::thread::spawn(move || worker.export(Format::Pdf).unwrap());
    let data = store.snapshot();
    let transaction = Transaction {
        id: TransactionId::new("edit").unwrap(),
        epoch: data.get().unwrap().epoch(),
        expected: vec![data.get().unwrap().dataset(DATA).unwrap().version()],
        operations: vec![Operation {
            dataset: DATA,
            mutation: Mutation::UpsertByKey(batch(&[(2, 0.5, Some(30.)), (6, 3., Some(9.))])),
        }],
    };
    assert!(matches!(
        store.apply(transaction),
        CommitOutcome::Applied(_)
    ));
    d.revision = Revision::new(9);
    p.annotation_revision = Revision::new(5);
    if let Primitive::Text { text, .. } = &mut p.annotations[0].primitive {
        *text = "Later annotation".into();
    }
    let later =
        FigureSnapshot::capture(&d, store.snapshot(), &ChartState::default(), fonts(), p).unwrap();
    assert_eq!(snapshot.export(Format::Svg).unwrap().bytes, before);
    assert_ne!(later.export(Format::Svg).unwrap().bytes, before);
    assert_eq!(
        thread.join().unwrap().metadata.stamp.store,
        Revision::INITIAL
    );
    assert_eq!(later.metadata().stamp.store, Revision::new(1));
    assert_eq!(
        snapshot.metadata().profile.annotation_revision,
        Revision::new(4)
    );
    drop(snapshot);
    assert!(weak_layout.upgrade().is_none());
    assert!(weak_bytes.upgrade().is_none());
}
#[test]
fn deterministic_preview_and_png_share_the_same_publication_geometry() {
    let mut p = profile();
    p.dpi = 72;
    let s = capture(p.clone());
    assert_eq!(
        s.export(Format::Svg).unwrap().bytes,
        s.export(Format::Svg).unwrap().bytes
    );
    assert_eq!(
        s.export(Format::Png).unwrap().bytes,
        s.export(Format::Png).unwrap().bytes
    );
    p.text = TextMode::Outline;
    let outlined = capture(p);
    assert_eq!(
        s.preview_svg().unwrap(),
        outlined.export(Format::Svg).unwrap().bytes
    );
    assert_eq!(
        s.layout().scene().items(),
        outlined.layout().scene().items()
    );
    let svg = String::from_utf8(s.preview_svg().unwrap()).unwrap();
    assert!(!svg.contains("<text"));
    assert!(!svg.contains("<image"));
}
#[test]
fn png_straight_alpha_is_not_premultiplied_or_double_composited() {
    let mut p = profile();
    p.page = PageSize::points(40., 40.).unwrap();
    p.dpi = 72;
    p.annotations.clear();
    p.background = Color {
        red: 200,
        green: 100,
        blue: 50,
        alpha: 128,
    }
    .into();
    p.layout.padding = 0.;
    p.layout.minimum_plot = (1., 1.);
    for a in &mut p.layout.axes {
        a.visible = false;
    }
    let d = ChartDefinition::new(Revision::INITIAL);
    let s = FigureSnapshot::capture(&d, store().snapshot(), &ChartState::default(), fonts(), p)
        .unwrap();
    let (_, data, _) = png_data(&s.export(Format::Png).unwrap().bytes);
    let pixel = &data[0..4];
    assert_eq!(pixel[3], 128);
    for (value, expected) in pixel[..3].iter().zip([200u8, 100, 50]) {
        assert!(value.abs_diff(expected) <= 1);
    }
}
fn font_with_permission(flags: u16) -> ChartResult<FontResource> {
    let mut bytes = FONT_BYTES.to_vec();
    let count = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    for i in 0..count {
        let table = 12 + i * 16;
        if &bytes[table..table + 4] == b"OS/2" {
            let offset =
                u32::from_be_bytes(bytes[table + 8..table + 12].try_into().unwrap()) as usize;
            bytes[offset + 8..offset + 10].copy_from_slice(&flags.to_be_bytes());
        }
    }
    let mut descriptor = font().descriptor();
    descriptor.revision = Revision::new(1);
    FontResource::new(descriptor, Arc::from(bytes))
}
#[test]
fn font_permissions_missing_glyphs_and_exact_resource_revisions_fail_with_context() {
    let mut p = profile();
    p.layout.font.revision = Revision::new(99);
    let e = FigureSnapshot::capture(
        &definition(),
        store().snapshot(),
        &ChartState::default(),
        fonts(),
        p,
    )
    .err()
    .unwrap();
    assert_eq!(e.code, DiagnosticCode::MissingResource);
    assert_eq!(e.context.resource_revision, Some(Revision::new(99)));
    let mut p = profile();
    if let Primitive::Text { text, .. } = &mut p.annotations[0].primitive {
        *text = "\u{10ffff}".into();
    }
    let e = FigureSnapshot::capture(
        &definition(),
        store().snapshot(),
        &ChartState::default(),
        fonts(),
        p,
    )
    .err()
    .unwrap();
    assert_eq!(e.code, DiagnosticCode::MissingResource);
    assert_eq!(e.context.resource, Some(ResourceId::new(0)));
    assert!(e.context.stamp.is_some());
    let restricted = font_with_permission(0x100).unwrap();
    let mut p = profile();
    p.layout.font = restricted.descriptor();
    let resources = FontResources::new(vec![restricted]).unwrap();
    let s = FigureSnapshot::capture(
        &definition(),
        store().snapshot(),
        &ChartState::default(),
        resources.clone(),
        p.clone(),
    )
    .unwrap();
    assert_eq!(
        s.export(Format::Pdf).unwrap_err().code,
        DiagnosticCode::ExportFidelity
    );
    assert!(s.export(Format::Svg).is_ok());
    p.text = TextMode::Outline;
    let s = FigureSnapshot::capture(
        &definition(),
        store().snapshot(),
        &ChartState::default(),
        resources,
        p,
    )
    .unwrap();
    assert!(s.export(Format::Pdf).is_ok());
}
#[test]
fn invalid_resources_units_precision_and_raster_budgets_reject_recoverably() {
    let mut descriptor = font().descriptor();
    descriptor.byte_len = 1;
    assert!(FontResource::new(descriptor, Arc::from(FONT_BYTES)).is_err());
    let mut p = profile();
    p.layout.units = Units::LogicalPixels;
    assert_eq!(
        FigureSnapshot::capture(
            &definition(),
            store().snapshot(),
            &ChartState::default(),
            fonts(),
            p
        )
        .err()
        .unwrap()
        .code,
        DiagnosticCode::UnsupportedCapability
    );
    let mut p = profile();
    p.max_raster_pixels = 10;
    let s = capture(p);
    assert_eq!(
        s.export(Format::Png).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert!(s.export(Format::Svg).is_ok());
    let mut p = profile();
    p.max_output_bytes = 32;
    assert_eq!(
        FigureSnapshot::capture(
            &definition(),
            store().snapshot(),
            &ChartState::default(),
            fonts(),
            p
        )
        .err()
        .unwrap()
        .code,
        DiagnosticCode::ResourceLimit
    );
    let mut p = profile();
    if let Primitive::Text { origin, .. } = &mut p.annotations[0].primitive {
        *origin = Point::new(1e16, 20.).unwrap();
    }
    assert_eq!(
        FigureSnapshot::capture(
            &definition(),
            store().snapshot(),
            &ChartState::default(),
            fonts(),
            p
        )
        .err()
        .unwrap()
        .code,
        DiagnosticCode::PrecisionLoss
    );
}
#[test]
fn plain_text_xml_escaping_curves_and_empty_clips_are_supported() {
    let mut p = profile();
    if let Primitive::Text { text, .. } = &mut p.annotations[0].primitive {
        *text = "A < B & C > D".into();
    }
    p.annotations.push(SceneItem {
        guide: None,
        layer: Some(LayerId::new(99)),
        clip: Some(Rect::new(0., 0., 0., 0.).unwrap()),
        primitive: Primitive::Path {
            commands: vec![
                PathCommand::MoveTo(Point::new(1., 1.).unwrap()),
                PathCommand::QuadraticTo(Point::new(2., 4.).unwrap(), Point::new(8., 8.).unwrap()),
                PathCommand::CubicTo(
                    Point::new(12., 10.).unwrap(),
                    Point::new(17., 19.).unwrap(),
                    Point::new(20., 22.).unwrap(),
                ),
                PathCommand::Close,
            ],
            stroke: Stroke {
                color: Color {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
                width: 1.,
            },
        },
    });
    p.dpi = 72;
    let empty = capture(p.clone());
    p.annotations.last_mut().unwrap().clip = None;
    let s = capture(p);
    let svg = String::from_utf8(s.export(Format::Svg).unwrap().bytes).unwrap();
    assert!(svg.contains("A &lt; B &amp; C &gt; D"));
    assert!(s.export(Format::Pdf).is_ok());
    assert!(empty.export(Format::Pdf).is_ok());
    assert_ne!(
        s.export(Format::Png).unwrap().bytes,
        empty.export(Format::Png).unwrap().bytes
    );
}

#[test]
fn restricted_and_preview_print_only_embedding_obey_permissions() {
    for flags in [0x0002, 0x0200, 0x8000] {
        assert_eq!(
            font_with_permission(flags).unwrap_err().code,
            DiagnosticCode::ExportFidelity
        );
    }
    let f = font_with_permission(4).unwrap();
    let mut p = profile();
    p.layout.font = f.descriptor();
    let fonts = FontResources::new(vec![f]).unwrap();
    let snapshot = FigureSnapshot::capture(
        &definition(),
        store().snapshot(),
        &ChartState::default(),
        fonts,
        p,
    )
    .unwrap();
    assert_eq!(
        snapshot.export(Format::Svg).unwrap_err().code,
        DiagnosticCode::ExportFidelity
    );
    assert!(snapshot.export(Format::Pdf).is_ok());
    assert_eq!(
        Format::Pdf.capabilities(TextMode::Preserve).text,
        TextRepresentation::EmbeddedSubsetFonts
    );
    assert!(!Format::Png.capabilities(TextMode::Preserve).vector_marks);
}
