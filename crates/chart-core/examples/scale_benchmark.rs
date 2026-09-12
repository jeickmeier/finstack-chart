//! SP-07 scale kernel measurements feeding WP-22; not the release budget gate.
use chart_core::{
    ChartResult,
    interpolate::{Number, Value},
    scales::*,
};
use std::{hint::black_box, time::Instant};
fn measure(mut run: impl FnMut(), operations: usize) -> serde_json::Value {
    run();
    let mut samples = vec![];
    for _ in 0..7 {
        let start = Instant::now();
        run();
        samples.push(start.elapsed().as_nanos() as f64 / operations as f64);
    }
    samples.sort_by(f64::total_cmp);
    serde_json::json!({"operations_per_sample":operations,"samples":7,"ns_per_operation_min":samples[0],"ns_per_operation_median":samples[3],"ns_per_operation_max":samples[6]})
}
fn main() -> ChartResult<()> {
    let mut records = vec![];
    for knots in [2, 128, 4096] {
        let mut spec = NumericScaleSpec::d3(NumericFamily::Linear);
        spec.domain = (0..knots).map(|i| Number(i as f64)).collect();
        spec.range = (0..knots).map(|i| Number((i * i) as f64)).collect();
        let scale = NumericScale::new(spec)?;
        let count = 200_000;
        let timing = measure(
            || {
                for i in 0..count {
                    black_box(
                        scale
                            .map_finite(black_box((i % (knots - 1)) as f64 + 0.25))
                            .unwrap(),
                    );
                }
            },
            count,
        );
        records
            .push(serde_json::json!({"kernel":"piecewise_forward","knots":knots,"timing":timing}));
    }
    for keys in [1000, 100_000] {
        let domain: Vec<_> = (0..keys)
            .map(|i| ScaleKey::Unsigned(9_007_199_254_740_992 + i))
            .collect();
        let scale = OrdinalScale::new(OrdinalSpec {
            domain: domain.clone(),
            range: vec![0, 1, 2],
            unknown: OrdinalUnknown::Implicit,
        });
        let count = 200_000;
        let timing = measure(
            || {
                for i in 0..count {
                    black_box(scale.map(black_box(&domain[i % domain.len()])).unwrap());
                }
            },
            count,
        );
        records
            .push(serde_json::json!({"kernel":"typed_ordinal_lookup","keys":keys,"timing":timing}));
    }
    for samples in [1000, 100_000] {
        let values: Vec<_> = (0..samples)
            .map(|i| Some(Number(((i * 65537) % samples) as f64)))
            .collect();
        let mapped = MappedScaleSpec {
            resolved_numeric_limits: None,
            limits_function: None,
            guide: None,
            ggplot: None,
            catalog: None,
            function: ScaleFunctionSpec::Classifier(ClassifierSpec {
                domain: ClassifierDomain::Quantile(vec![]),
                range: (0..7).map(|i| Value::number(f64::from(i))).collect(),
                unknown: None,
            }),
            training: ScaleTraining::Eligible,
        };
        let count = 20;
        let timing = measure(
            || {
                for _ in 0..count {
                    black_box(
                        MappedScale::for_numbers(mapped.trained(black_box(&values)).unwrap())
                            .unwrap(),
                    );
                }
            },
            count,
        );
        records.push(serde_json::json!({"kernel":"eligible_quantile_retrain_prepare","samples":samples,"buckets":7,"timing":timing}));
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"profile":"release","method":"one warmup; seven samples; allocation included in retraining, inputs prepared outside timing","measurements":records})).unwrap());
    Ok(())
}
