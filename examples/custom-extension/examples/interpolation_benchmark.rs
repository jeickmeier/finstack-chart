//! ITP-08 finite preparation/sampling/allocation profile; not a release performance gate.
use chart_core::{ChartResult, Revision, grammar::OperationRef, interpolate::*};
use std::{hint::black_box, time::Instant};

// Instrumentation-only allocator shim. Every operation forwards the exact allocation
// contract to System; production libraries retain the workspace unsafe-code prohibition.
#[allow(unsafe_code)]
mod allocation {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed};
    pub struct Counting;
    static ENABLED: AtomicBool = AtomicBool::new(false);
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    static BYTES: AtomicUsize = AtomicUsize::new(0);
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if ENABLED.load(Relaxed) {
                CALLS.fetch_add(1, Relaxed);
                BYTES.fetch_add(layout.size(), Relaxed);
            }
            // SAFETY: forward the caller's unchanged valid layout to System.
            unsafe { System.alloc(layout) }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            // SAFETY: all allocations use System and the caller supplies the matching layout.
            unsafe { System.dealloc(ptr, layout) }
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            if ENABLED.load(Relaxed) {
                CALLS.fetch_add(1, Relaxed);
                BYTES.fetch_add(size, Relaxed);
            }
            // SAFETY: forward the unchanged allocation and requested size to System.
            unsafe { System.realloc(ptr, layout, size) }
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            if ENABLED.load(Relaxed) {
                CALLS.fetch_add(1, Relaxed);
                BYTES.fetch_add(layout.size(), Relaxed);
            }
            // SAFETY: forward the caller's unchanged valid layout to System.
            unsafe { System.alloc_zeroed(layout) }
        }
    }
    pub fn start() {
        CALLS.store(0, Relaxed);
        BYTES.store(0, Relaxed);
        ENABLED.store(true, Relaxed);
    }
    pub fn stop() -> (usize, usize) {
        ENABLED.store(false, Relaxed);
        (CALLS.load(Relaxed), BYTES.load(Relaxed))
    }
}
#[global_allocator]
static ALLOCATOR: allocation::Counting = allocation::Counting;
fn measure(name: &str, count: usize, mut run: impl FnMut()) -> serde_json::Value {
    let start = Instant::now();
    run();
    let cold = start.elapsed().as_nanos();
    for _ in 0..10 {
        run();
    }
    let mut raw = Vec::with_capacity(30);
    for _ in 0..30 {
        let start = Instant::now();
        run();
        raw.push(start.elapsed().as_nanos() as f64 / count as f64);
    }
    allocation::start();
    run();
    let (allocations, bytes) = allocation::stop();
    let mut sorted = raw.clone();
    sorted.sort_by(f64::total_cmp);
    serde_json::json!({"workload":name,"operations_per_sample":count,"cold_batch_ns":cold.to_string(),"warmups":10,"raw_ns_per_operation":raw,"p50_ns":sorted[14],"p95_ns":sorted[28],"allocation_pass":{"calls":allocations,"requested_bytes":bytes,"calls_per_operation":allocations as f64/count as f64,"requested_bytes_per_operation":bytes as f64/count as f64}})
}
fn main() -> ChartResult<()> {
    let mut specs = vec![
        (
            "number".to_owned(),
            InterpolationSpec::Between {
                factory: InterpolationFactory::new(FactoryKind::Number),
                a: Value::number(-12.),
                b: Value::number(150.),
            },
        ),
        (
            "lab".to_owned(),
            InterpolationSpec::Between {
                factory: InterpolationFactory::new(FactoryKind::Lab),
                a: Value::Text("#d54a3a".into()),
                b: Value::Text("#28689b".into()),
            },
        ),
        (
            "transform".to_owned(),
            InterpolationSpec::TransformText {
                a: "rotate(-25) scale(0.7)".into(),
                b: "translate(40,10) rotate(65) scale(1.3)".into(),
                syntax: TransformSyntax::Svg,
            },
        ),
        (
            "zoom".to_owned(),
            InterpolationSpec::Zoom {
                a: ZoomView::new([0., 0., 100.])?,
                b: ZoomView::new([40., 20., 50.])?,
                rho: None,
            },
        ),
    ];
    for size in [16, 1024] {
        let array = |offset: f64| {
            Value::Array(
                (0..size)
                    .map(|i| {
                        Value::Array(vec![
                            Value::number(i as f64 + offset),
                            Value::Text(format!("{}px", i as f64 + offset)),
                        ])
                    })
                    .collect(),
            )
        };
        specs.push((
            format!("nested-{size}"),
            InterpolationSpec::Between {
                factory: InterpolationFactory::new(FactoryKind::Value),
                a: array(0.),
                b: array(5.),
            },
        ));
    }
    let registry = chart_extension_example::registry()?;
    specs.push((
        "registered-number".into(),
        InterpolationSpec::Between {
            factory: registry.interpolation_factory(
                OperationRef::new("example.interpolation", Revision::new(1)),
                serde_json::json!({"mode":"SquaredNumber"}),
            )?,
            a: Value::number(0.),
            b: Value::number(100.),
        },
    ));
    let mut records = vec![];
    for (name, spec) in specs {
        let count = if name == "nested-1024" { 20 } else { 200 };
        records.push(measure(&format!("{name}/prepare"), count, || {
            for _ in 0..count {
                black_box(
                    Interpolator::new_with_registry(black_box(spec.clone()), &registry).unwrap(),
                );
            }
        }));
        let f = Interpolator::new_with_registry(spec, &registry)?;
        records.push(measure(&format!("{name}/sample_owned"), count, || {
            for i in 0..count {
                black_box(f.sample(black_box((i % 101) as f64 / 100.)).unwrap());
            }
        }));
        let mut destination = f.sample(0.)?;
        records.push(measure(&format!("{name}/sample_into"), count, || {
            for i in 0..count {
                f.sample_into(black_box((i % 101) as f64 / 100.), &mut destination)
                    .unwrap();
                black_box(&destination);
            }
        }));
        if name == "lab" {
            records.push(measure("lab/sample_color", count, || {
                for i in 0..count {
                    black_box(f.sample_color(black_box((i % 101) as f64 / 100.)).unwrap());
                }
            }));
        }
        if name == "transform" {
            records.push(measure("transform/sample_matrix", count, || {
                for i in 0..count {
                    black_box(
                        f.sample_transform(black_box((i % 101) as f64 / 100.))
                            .unwrap(),
                    );
                }
            }));
        }
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"profile":"release","protocol":"ADR-008 finite profile; monotonic Instant, 10 warmups and 30 measured batches, nearest-rank quantiles; cold batch separate. Allocation counts from one separate warm batch, counting alloc/alloc_zeroed/realloc and requested bytes, not live bytes or RSS. No native/GPU/streaming claims.","seed":"deterministic explicit endpoints; sampling i modulo 101 / 100","allocator":"System with instrumentation shim; single-thread workload","measurements":records})).unwrap());
    Ok(())
}
