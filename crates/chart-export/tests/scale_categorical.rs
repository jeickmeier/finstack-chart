//! SP-03 named D3 category axes, dodge slots and live retained publication.
use chart_core::plot::{scale_band_d3, scale_point_d3};
use chart_core::{grammar::GroupValue, prelude::*, scales::*, scene::Primitive};
use chart_export::*;
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
fn positions(f: &FigureSnapshot) -> Vec<(f64, f64)> {
    f.scene()
        .items()
        .iter()
        .filter_map(|i| match i.primitive {
            Primitive::Point { center, .. } if i.layer.is_some() => Some((center.x(), center.y())),
            _ => None,
        })
        .collect()
}
#[test]
fn named_category_axes_preserve_dodge_slots_and_descriptor_versions() {
    for point in [false, true] {
        for reverse in [false, true] {
            let axis_scale = if point {
                scale_point_d3(PointSpec {
                    round: true,
                    align: 0.,
                    ..PointSpec::default()
                })
            } else {
                scale_band_d3(BandSpec {
                    padding_inner: 0.5,
                    round: true,
                    ..BandSpec::default()
                })
            };
            let axis = x_axis()
                .name("category")
                .scale(axis_scale.categories(["a", "a", "b"]))
                .visible(false);
            let axis = if reverse {
                axis.range(210., 110.)
            } else {
                axis.range(110., 210.)
            };
            let id = axis.handle().unwrap().id();
            let layer = points().axes("category", "y");
            let layer = if point {
                layer
            } else {
                layer.position(dodge(vec![
                    GroupValue::Int(0),
                    GroupValue::Int(1),
                    GroupValue::Int(2),
                ]))
            };
            let p = plot(
                Data::columns()
                    .column("x", categorical(["a", "a", "b"]))
                    .column("y", [1., 2., 3.])
                    .column("g", [0_i64, 2, 0])
                    .build()
                    .unwrap(),
            )
            .aes(aes().x("x").y("y").group("g"))
            .x_axis(x_axis().visible(false))
            .axis(axis)
            .layer(layer)
            .build()
            .unwrap();
            // Explicit axis ranges make the independent expected pixel positions invariant
            // to publication margins and prove reversed band orientation through dodge.
            let mut wire: serde_json::Value = serde_json::from_str(&p.to_json().unwrap()).unwrap();
            assert_eq!(wire["version"], 5);
            // The authoring API retains the new axis in its portable semantic definition.
            let roundtrip = Plot::from_json(&wire.to_string()).unwrap();
            assert_eq!(roundtrip.to_json().unwrap(), p.to_json().unwrap());
            wire["version"] = 4.into();
            assert!(Plot::from_json(&wire.to_string()).is_err());
            let f = Output::new(FONT)
                .unwrap()
                .request(&p, export_options(PageSize::points(400., 300.).unwrap()))
                .unwrap()
                .prepare()
                .unwrap();
            let actual = positions(&f);
            assert_eq!(actual.len(), 3);
            let expected = if point {
                if reverse {
                    [210., 210., 110.]
                } else {
                    [110., 110., 210.]
                }
            } else {
                // 100 / (2-.5) floors to step 66; band width rounds to 33,
                // leftover alignment rounds the first physical start to 111.
                let centers = if reverse {
                    [193.5, 193.5, 127.5]
                } else {
                    [127.5, 127.5, 193.5]
                };
                let delta = if reverse { -11. } else { 11. };
                [centers[0] - delta, centers[1] + delta, centers[2] - delta]
            };
            for (p, e) in actual.iter().zip(expected) {
                assert!(
                    (p.0 - e).abs() < 1e-10,
                    "point={point}, reverse={reverse}: {} != {e}",
                    p.0
                );
            }
            let axis = &f.layout().axes()[&id];
            let capabilities = match &axis.scale {
                chart_core::layout::ResolvedScale::Band(s) => s.capabilities(),
                chart_core::layout::ResolvedScale::Point(s) => s.capabilities(),
                _ => panic!("wrong category family"),
            };
            assert!(!capabilities.numeric_inverse);
            assert!(capabilities.category_lookup);
            assert!(!f.export(Format::Svg).unwrap().bytes.is_empty());
        }
    }
}

fn data(labels: Vec<&str>) -> Data {
    let n = labels.len();
    Data::columns()
        .column("x", categorical(labels))
        .column("y", vec![1.; n])
        .build()
        .unwrap()
}
#[test]
fn category_append_matches_fresh_batch_and_retains_prior_export() {
    let build = |d| {
        plot(d)
            .aes(aes().x("x").y("y"))
            .x_axis(
                x_axis()
                    .scale(scale_band_d3(BandSpec::default()))
                    .range(100., 300.),
            )
            .layer(points())
            .build()
            .unwrap()
    };
    let initial = build(data(vec!["b", "a"]));
    let mut chart = Chart::new(initial).unwrap();
    let output = Output::new(FONT).unwrap();
    let options =
        export_options(PageSize::points(400., 300.).unwrap()).basis(CaptureBasis::Current);
    let old = output
        .live_request(&chart, options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let old_positions = positions(&old);
    let old_svg = old.export(Format::Svg).unwrap().bytes;
    let transaction = chart
        .transaction()
        .unwrap()
        .append("data", data(vec!["c"]))
        .build()
        .unwrap();
    chart.apply_transaction(transaction).unwrap();
    let current = output
        .live_request(&chart, options.clone())
        .unwrap()
        .prepare()
        .unwrap();
    let fresh = output
        .request(&build(data(vec!["b", "a", "c"])), options)
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(positions(&current), positions(&fresh));
    assert_eq!(positions(&old), old_positions);
    assert_eq!(old.export(Format::Svg).unwrap().bytes, old_svg);
    assert_ne!(positions(&current), old_positions);
}
