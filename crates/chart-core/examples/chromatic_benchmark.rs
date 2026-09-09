//! FIX-21 component measurements consumed by WP-22, without a release budget claim.
use chart_core::{ChartResult, scales::chromatic::*};
use std::{hint::black_box, time::Instant};
fn measure(mut work: impl FnMut(), n: usize) -> serde_json::Value {
    work();
    let mut samples = vec![];
    for _ in 0..7 {
        let start = Instant::now();
        work();
        samples.push(start.elapsed().as_nanos() as f64 / n as f64);
    }
    samples.sort_by(f64::total_cmp);
    serde_json::json!({"operations_per_sample":n,"samples":7,"ns_min":samples[0],"ns_median":samples[3],"ns_max":samples[6]})
}
fn main() -> ChartResult<()> {
    let mut records = vec![];
    let n = 50_000;
    for id in [
        InterpolatorId::Viridis,
        InterpolatorId::Blues,
        InterpolatorId::RdBu,
        InterpolatorId::Rainbow,
        InterpolatorId::Sinebow,
    ] {
        let spec = ChromaticSpec { id, reverse: false };
        records.push(serde_json::json!({"kernel":"prepare","id":id,"timing":measure(||{for _ in 0..n {black_box(ChromaticRamp::new(black_box(spec)).unwrap());}},n)}));
        let ramp = ChromaticRamp::new(spec)?;
        records.push(serde_json::json!({"kernel":"evaluate","id":id,"timing":measure(||{for i in 0..n {black_box(ramp.evaluate(black_box((i%4096) as f64/4095.)).unwrap());}},n)}));
    }
    // Large heatmap-like color workloads reuse the same prepared table and positional data.
    for rows in [1_000usize, 10_000, 100_000] {
        use chart_core::{
            grammar::Compiler, interpolate::InterpolationSpec, prelude::*, scales::*,
            state::ChartState,
        };
        let mapping = |id| {
            ScaleConstructor::Sequential
                .create(ScaleOptions {
                    interpolator: Some(InterpolationSpec::Chromatic {
                        spec: ChromaticSpec { id, reverse: false },
                    }),
                    ..Default::default()
                })
                .unwrap()
                .mapped(ScaleTraining::Authored)
                .unwrap()
        };
        let p = plot(
            Data::columns()
                .column(
                    "x",
                    (0..rows)
                        .map(|i| i as f64 / rows as f64)
                        .collect::<Vec<_>>(),
                )
                .column("y", vec![1.; rows])
                .build()?,
        )
        .aes(aes().x("x").y("y").color("x").color_scale("palette"))
        .layer(points())
        .scale(color_mapped("palette", mapping(InterpolatorId::Viridis)))
        .build()?;
        let source = p.source();
        let state = ChartState::default();
        let mut compiler = Compiler::new();
        let first = compiler.prepare(p.definition(), &source, &state, p.compile_limits())?;
        let mut d = p.definition().clone();
        let mut revision = 0;
        let count = 10;
        let timing = measure(
            || {
                for _ in 0..count {
                    revision += 1;
                    d.revision = chart_core::Revision::new(revision);
                    let ColorScale::Mapped { scale, .. } =
                        &mut d.layers[0].color.as_mut().unwrap().scale
                    else {
                        unreachable!()
                    };
                    *scale = mapping(if revision % 2 == 0 {
                        InterpolatorId::Viridis
                    } else {
                        InterpolatorId::Blues
                    });
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
            count,
        );
        records.push(serde_json::json!({"kernel":"palette_only_prepare","rows":rows,"statistics_reused":true,"timing":timing}));
    }
    let mut arrays = 0;
    let mut colors = 0;
    for id in SchemeId::ALL {
        let info = id.info();
        for &size in info.sizes {
            arrays += 1;
            colors += scheme(
                id,
                if info.family == SchemeFamily::Categorical {
                    None
                } else {
                    Some(size)
                },
                false,
            )?
            .len();
        }
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"profile":"release","method":"one warmup and seven samples; precompiled output sampling; inputs excluded","catalog":{"schemes":38,"ramps":38,"discrete_arrays":arrays,"discrete_rgb_colors":colors,"discrete_rgb_payload_bytes":colors*4,"lookup_rgb_payload_bytes":4*256*4,"scope":"literal u32 payload only; excludes metadata, instructions and allocator overhead"},"measurements":records})).unwrap());
    Ok(())
}
