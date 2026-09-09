//! CLR-05 component timings for WP-22; no release budget is inferred here.
use chart_core::{
    ChartResult, Revision,
    color::{self, ColorSpace, ColorValue},
    grammar::Compiler,
    interpolate::{FactoryKind, InterpolationFactory, Value},
    prelude::*,
    state::ChartState,
};
use std::{hint::black_box, time::Instant};
fn measure(mut run: impl FnMut(), count: usize) -> serde_json::Value {
    run();
    let mut samples = vec![];
    for _ in 0..7 {
        let start = Instant::now();
        run();
        samples.push(start.elapsed().as_nanos() as f64 / count as f64);
    }
    samples.sort_by(f64::total_cmp);
    serde_json::json!({"operations_per_sample":count,"samples":7,"ns_min":samples[0],"ns_median":samples[3],"ns_max":samples[6]})
}
fn main() -> ChartResult<()> {
    let count = 50_000;
    let mut records = vec![];
    records.push(serde_json::json!({"kernel":"parse_css","timing":measure(|| { for _ in 0..count { black_box(color::color(black_box("rgba(12.3%, 40%, 98%, 0.37)")).unwrap()); } },count)}));
    let rgb = ColorValue::from(color::rgb(25.5, 102., 249.9).opacity(0.37));
    records.push(serde_json::json!({"kernel":"rgb_lab_rgb","timing":measure(|| { for _ in 0..count { black_box(black_box(rgb).convert(ColorSpace::Lab).convert(ColorSpace::Rgb)); } },count)}));
    let hcl = rgb.convert(ColorSpace::Hcl);
    records.push(serde_json::json!({"kernel":"hcl_to_paint","timing":measure(|| { for _ in 0..count { black_box(black_box(hcl).to_paint()); } },count)}));
    let factory = InterpolationFactory::new(FactoryKind::Hcl);
    let a = Value::Color(rgb);
    let b = Value::Color(color::rgb(250., 20., 60.).into());
    records.push(serde_json::json!({"kernel":"prepare_hcl_interpolator","timing":measure(|| { for _ in 0..count { black_box(factory.between(black_box(a.clone()),black_box(b.clone())).unwrap()); } },count)}));
    let interpolator = factory.between(a, b)?;
    records.push(serde_json::json!({"kernel":"sample_prepared_hcl","timing":measure(|| { for i in 0..count { black_box(interpolator.sample(black_box((i%100) as f64/100.)).unwrap()); } },count)}));
    for rows in [1000, 10_000] {
        let p = plot(
            Data::columns()
                .column("x", (0..rows).map(|i| i as f64).collect::<Vec<_>>())
                .column("y", vec![1.; rows])
                .build()?,
        )
        .aes(aes().x("x").y("y"))
        .layer(points())
        .build()?;
        let source = p.source();
        let state = ChartState::default();
        let mut compiler = Compiler::new();
        let first = compiler.prepare(p.definition(), &source, &state, p.compile_limits())?;
        let mut d = p.definition().clone();
        let mut revision = 0;
        let updates = 20;
        let timing = measure(
            || {
                for _ in 0..updates {
                    revision += 1;
                    d.revision = Revision::new(revision);
                    d.layers[0].style.color = color::hcl((revision % 360) as f64, 65., 60.).into();
                    let next = compiler
                        .prepare(&d, &source, &state, p.compile_limits())
                        .unwrap();
                    assert_eq!(next.metrics().evaluated_layers, 0);
                    assert!(std::sync::Arc::ptr_eq(
                        first.layers()[0].table(),
                        next.layers()[0].table()
                    ));
                    black_box(next);
                }
            },
            updates,
        );
        records.push(serde_json::json!({"kernel":"color_only_prepare","rows":rows,"statistics_reused":true,"timing":timing}));
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"profile":"release","method":"one warmup and seven samples; input generation excluded; prepared color update includes encoding and cache-reuse assertions","measurements":records})).unwrap());
    Ok(())
}
