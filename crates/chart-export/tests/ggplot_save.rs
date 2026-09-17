//! FIX-GG17 independent reference dimensions and explicit host write/custom devices.
use chart_core::{ChartResult, DiagnosticCode, prelude::*};
use chart_export::*;
use std::sync::Arc;
#[test]
fn reference_dimensions_and_filename_policy() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/device-controls.json"
    ))
    .unwrap();
    let mut passed = 0;
    let mut rejected = 0;
    for case in fixture["dimensions"].as_array().unwrap() {
        let options: SaveOptions = serde_json::from_value(case["input"].clone()).unwrap();
        let result = options.resolve("figure.PNG", None, 1);
        if case["result"].get("error").is_some() {
            assert!(result.is_err());
            rejected += 1;
        } else {
            let plan = result.unwrap();
            let expected = case["result"].as_array().unwrap();
            assert!((plan.page.width() / 72. - expected[0].as_f64().unwrap()).abs() < 1e-12);
            assert!((plan.page.height() / 72. - expected[1].as_f64().unwrap()).abs() < 1e-12);
            assert_eq!(plan.device, "png");
            passed += 1;
        }
    }
    assert_eq!((passed, rejected), (24, 2));
    let options = SaveOptions {
        width: Some(10.),
        units: SaveUnits::Centimeters,
        scale: 2.,
        ..Default::default()
    };
    let plan = options
        .resolve(
            "fig%%-%03d.JpG",
            Some(PageSize::points(100., 200.).unwrap()),
            7,
        )
        .unwrap();
    assert_eq!(plan.path.to_str().unwrap(), "fig%-007.JpG");
    assert_eq!(plan.device, "jpeg");
    assert_eq!(plan.page.height(), 400.);
    assert!(options.resolve("x.png", None, 1).is_err());
    assert!(options.resolve("x%q.png", Some(plan.page), 1).is_err());
    assert!(options.resolve("x.png", Some(plan.page), 0).is_err());
}
#[derive(Debug)]
struct Custom(bool);
impl CustomDevice for Custom {
    fn name(&self) -> &str {
        "example.raw"
    }
    fn encode(&self, figure: &FigureSnapshot, _: usize) -> ChartResult<Vec<u8>> {
        if self.0 {
            return Err(chart_core::Diagnostic::error(
                DiagnosticCode::NumericalDomain,
                "custom failure",
                "caller diagnostic",
            ));
        }
        Ok(format!(
            "{}x{}",
            figure.metadata().profile.page.width(),
            figure.metadata().profile.page.height()
        )
        .into_bytes())
    }
}
#[test]
fn custom_errors_budget_capture_and_explicit_directory_creation() {
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("y", [2., 3.])
        .build()
        .unwrap();
    let plot = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .build()
        .unwrap();
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let dir = std::env::temp_dir().join(format!("chart-save-{}", std::process::id()));
    let path = dir.join("new/figure.raw");
    let options = SaveOptions {
        width: Some(2.),
        height: Some(1.),
        device: Some("example.raw".into()),
        ..Default::default()
    };
    let plan = options.resolve(&path, None, 1).unwrap();
    let frame = output
        .request(&plot, plan.options(export_options(plan.page)))
        .unwrap()
        .prepare()
        .unwrap();
    let mut registry = DeviceRegistry::default();
    registry.register(Arc::new(Custom(false))).unwrap();
    let pinned = registry.clone();
    assert!(registry.register(Arc::new(Custom(true))).is_err());
    drop(registry);
    assert_eq!(plan.encode(&frame, &pinned, 100).unwrap(), b"144x72");
    assert!(plan.encode(&frame, &pinned, 2).is_err());
    assert!(plan.write(b"bytes").is_err());
    assert!(!dir.exists());
    let mut failing = DeviceRegistry::default();
    failing.register(Arc::new(Custom(true))).unwrap();
    assert_eq!(
        plan.encode(&frame, &failing, 100).unwrap_err().code,
        DiagnosticCode::NumericalDomain
    );
    let creating = SaveOptions {
        create_dir: true,
        ..options
    }
    .resolve(&path, None, 1)
    .unwrap();
    creating
        .write(&creating.encode(&frame, &pinned, 100).unwrap())
        .unwrap();
    assert_eq!(std::fs::read(path).unwrap(), b"144x72");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn retained_pdf_pages_preserve_single_page_bytes_and_distinct_pages() {
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let first = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .title(title("FIRST PAGE"))
        .build()
        .unwrap();
    let second = first.edit().title(title("SECOND PAGE")).build().unwrap();
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let options = export_options(PageSize::points(200., 120.).unwrap());
    let a = output
        .request(&first, options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let b = output.request(&second, options).unwrap().prepare().unwrap();
    assert_eq!(
        FigurePages::new(a.clone()).export(Format::Pdf).unwrap(),
        a.export(Format::Pdf).unwrap().bytes
    );
    let mut pages = FigurePages::new(a.clone());
    pages.push(b.clone()).unwrap();
    let mut expected_pages = FigurePages::new(a);
    expected_pages.push(b).unwrap();
    let expected = expected_pages.export(Format::Pdf).unwrap();
    drop(output);
    drop(first);
    drop(second);
    assert_eq!(pages.export(Format::Pdf).unwrap(), expected);
    assert!(expected.starts_with(b"%PDF-"));
    assert!(pages.export(Format::Eps).is_err());
    assert!(pages.export(Format::Png).is_err());
}

#[test]
fn custom_device_cannot_exceed_captured_output_budget() {
    #[derive(Debug)]
    struct Observed(std::sync::Arc<std::sync::atomic::AtomicUsize>);
    impl CustomDevice for Observed {
        fn name(&self) -> &str {
            "observed"
        }
        fn encode(&self, _: &FigureSnapshot, limit: usize) -> chart_core::ChartResult<Vec<u8>> {
            self.0.store(limit, std::sync::atomic::Ordering::Relaxed);
            Ok(vec![0; 4_194_305])
        }
    }
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let plot = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .build()
        .unwrap();
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let plan = SaveOptions {
        width: Some(2.),
        height: Some(1.),
        device: Some("observed".into()),
        ..Default::default()
    }
    .resolve("figure.raw", None, 1)
    .unwrap();
    let frame = output
        .request(
            &plot,
            plan.options(export_options(plan.page))
                .max_output_bytes(4_194_304),
        )
        .unwrap()
        .prepare()
        .unwrap();
    let observed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut registry = DeviceRegistry::default();
    registry
        .register(Arc::new(Observed(observed.clone())))
        .unwrap();
    assert_eq!(
        plan.encode(&frame, &registry, 8_388_608).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(
        observed.load(std::sync::atomic::Ordering::Relaxed),
        4_194_304
    );
    assert_eq!(
        plan.encode(&frame, &registry, 16).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(observed.load(std::sync::atomic::Ordering::Relaxed), 16);
}
