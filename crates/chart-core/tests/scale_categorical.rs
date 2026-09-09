//! SP-03 / FIX-20 category order, generic output and prepared spacing contracts.
use chart_core::{interpolate::Number, scales::*};
use serde_json::Value as Json;
fn key(v: &Json) -> ScaleKey {
    match v {
        Json::Null => ScaleKey::Null,
        Json::Bool(v) => ScaleKey::Boolean(*v),
        Json::String(v) => ScaleKey::Text(v.clone()),
        Json::Number(v) => ScaleKey::Number(Number(v.as_f64().unwrap())),
        _ => panic!("unexpected oracle key {v}"),
    }
}
fn optional(c: &Json, field: &str, default: f64) -> f64 {
    c[field][0].as_f64().unwrap_or(default)
}
fn close(a: f64, e: &Json, exact: bool) {
    let e = e.as_f64().unwrap();
    if exact {
        assert_eq!(a, e);
    } else {
        assert!((a - e).abs() <= 1e-12 + 1e-12 * e.abs(), "{a} != {e}");
    }
}
#[test]
fn all_pinned_band_point_and_ordinal_cases_match() {
    let corpus: Json =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-scale/cases.json")).unwrap();
    let mut counts = [0; 3];
    for c in corpus["cases"].as_array().unwrap() {
        let family = c["factory"].as_str().unwrap();
        if !matches!(family, "scaleBand" | "scalePoint" | "scaleOrdinal") {
            continue;
        }
        let config = &c["config"];
        let getters = &c["getters"];
        let domain = config["domain"][0]
            .as_array()
            .map(|v| v.iter().map(key).collect());
        if family == "scaleOrdinal" {
            counts[2] += 1;
            let spec = OrdinalSpec {
                domain: domain.unwrap_or_default(),
                range: config["range"][0].as_array().cloned().unwrap_or_default(),
                unknown: if config.get("unknown").is_some() {
                    OrdinalUnknown::Explicit(Some(config["unknown"][0].clone()))
                } else {
                    OrdinalUnknown::Implicit
                },
            };
            let mut s = OrdinalScale::new(spec);
            let empty = s.clone();
            for (i, v) in c["inputs"].as_array().unwrap().iter().enumerate() {
                let k = key(v);
                s = s.train([k.clone()]);
                let expected = &c["output"][i]["value"];
                assert_eq!(
                    s.map(&k),
                    (expected["kind"] != "Undefined").then_some(expected),
                    "{}",
                    c["id"]
                );
            }
            assert_eq!(
                s.spec().domain,
                getters["domain"]["value"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(key)
                    .collect::<Vec<_>>(),
                "{}",
                c["id"]
            );
            let copy = s.reconfigure(OrdinalSpec {
                domain: vec![],
                ..s.spec().clone()
            });
            assert!(copy.spec().domain.is_empty());
            assert_eq!(
                empty.spec().domain,
                OrdinalScale::new(empty.spec().clone()).spec().domain
            );
            continue;
        }
        let range = &getters["range"]["value"];
        let range = Bounds::new(range[0].as_f64().unwrap(), range[1].as_f64().unwrap()).unwrap();
        let round = config["round"][0] == true || config.get("rangeRound").is_some();
        let padding = optional(config, "padding", 0.);
        let align = optional(config, "align", 0.5);
        let s = if family == "scaleBand" {
            counts[0] += 1;
            CategoryScale::band(
                BandSpec {
                    domain,
                    padding_inner: optional(config, "paddingInner", padding),
                    padding_outer: optional(config, "paddingOuter", padding),
                    align,
                    round,
                },
                range,
            )
        } else {
            counts[1] += 1;
            CategoryScale::point(
                PointSpec {
                    domain,
                    padding,
                    align,
                    round,
                },
                range,
            )
        }
        .unwrap_or_else(|e| panic!("{}: {e}", c["id"]));
        assert_eq!(
            s.domain(),
            getters["domain"]["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(key)
                .collect::<Vec<_>>(),
            "{}",
            c["id"]
        );
        close(s.step(), &getters["step"]["value"], round);
        close(s.bandwidth(), &getters["bandwidth"]["value"], round);
        close(s.spec().align, &getters["align"]["value"], true);
        close(
            s.spec().padding_outer,
            if family == "scaleBand" {
                &getters["paddingOuter"]["value"]
            } else {
                &getters["padding"]["value"]
            },
            true,
        );
        if family == "scaleBand" {
            close(
                s.spec().padding_inner,
                &getters["paddingInner"]["value"],
                true,
            );
        }
        for (i, v) in c["inputs"].as_array().unwrap().iter().enumerate() {
            let output = s.map(&key(v)).unwrap();
            let expected = &c["output"][i]["value"];
            if expected["kind"] == "Undefined" {
                assert!(output.is_none(), "{}", c["id"]);
            } else {
                close(output.unwrap(), expected, round);
            }
        }
        let mut copied = s.spec().clone();
        copied.domain = Some(vec![]);
        assert!(
            CategoryScale::band(copied, range)
                .unwrap()
                .domain()
                .is_empty()
        );
        assert_eq!(s.clone(), s);
    }
    assert_eq!(counts, [62, 49, 19]);
}
#[test]
fn exact_typed_keys_and_explicit_training_do_not_mutate_frozen_catalogs() {
    let keys = vec![
        ScaleKey::Unsigned(u64::MAX),
        ScaleKey::Unsigned(u64::MAX - 1),
        ScaleKey::Integer(i64::MIN),
        ScaleKey::Timestamp(i64::MAX),
        ScaleKey::Number(Number(-0.)),
        ScaleKey::Number(Number(0.)),
        ScaleKey::Number(Number(f64::NAN)),
        ScaleKey::Number(Number(-f64::NAN)),
        ScaleKey::Text("0".into()),
        ScaleKey::Boolean(false),
        ScaleKey::Null,
    ];
    let s = OrdinalScale::new(OrdinalSpec {
        domain: vec![],
        range: vec![vec![1, 2], vec![3, 4]],
        unknown: OrdinalUnknown::Implicit,
    });
    let trained = s.train(keys.clone());
    assert!(s.spec().domain.is_empty());
    assert!(s.map(&keys[0]).is_none());
    assert_eq!(trained.spec().domain.len(), 9);
    assert_eq!(trained.map(&keys[0]), Some(&vec![1, 2]));
    assert_eq!(trained.map(&keys[1]), Some(&vec![3, 4]));
    assert_eq!(trained.map(&keys[4]), trained.map(&keys[5]));
    let decoded: OrdinalSpec<ScaleKey, Vec<i32>> =
        serde_json::from_str(&serde_json::to_string(trained.spec()).unwrap()).unwrap();
    assert_eq!(OrdinalScale::new(decoded), trained);
    let explicit = trained.reconfigure(OrdinalSpec {
        unknown: OrdinalUnknown::Explicit(Some(vec![9])),
        ..trained.spec().clone()
    });
    assert_eq!(explicit.train([ScaleKey::Text("new".into())]), explicit);
    assert_eq!(explicit.map(&ScaleKey::Text("new".into())), Some(&vec![9]));
    let bands = CategoryScale::band(
        BandSpec {
            domain: Some(keys),
            ..BandSpec::default()
        },
        Bounds::new(0., 90.).unwrap(),
    )
    .unwrap();
    assert_eq!(bands.domain().len(), 9);
    assert_eq!(bands.map(&ScaleKey::Unsigned(u64::MAX)).unwrap(), Some(0.));
    assert_eq!(
        bands.map(&ScaleKey::Unsigned(u64::MAX - 1)).unwrap(),
        Some(10.)
    );
}
#[test]
fn independent_spacing_windows_and_hit_geometry_preserve_category_identity() {
    let range = Bounds::new(0., 100.).unwrap();
    let labels = vec!["a".into()];
    let legacy = BandScale::resolve(
        &labels,
        &BandOptions {
            domain: None,
            inner_padding: 0.5,
            outer_padding: 0.,
        },
        range,
    )
    .unwrap();
    let d3 = BandScale::resolve_d3(
        &labels,
        &BandSpec {
            padding_inner: 0.5,
            ..BandSpec::default()
        },
        range,
    )
    .unwrap();
    assert_eq!(legacy.center("a").unwrap(), Some(25.));
    assert_eq!(d3.start("a").unwrap(), Some(25.));
    assert_eq!(d3.center("a").unwrap(), Some(50.));
    assert_eq!(d3.bandwidth(), 50.);
    assert_eq!(d3.category_at(50.).unwrap(), Some("a"));
    assert_eq!(d3.category_at(24.).unwrap(), None);
    for reverse in [false, true] {
        for round in [false, true] {
            let range = if reverse {
                Bounds::new(103., 0.).unwrap()
            } else {
                Bounds::new(0., 103.).unwrap()
            };
            let labels = vec!["a".into(), "b".into(), "c".into(), "d".into()];
            let bands = BandScale::resolve_d3(
                &labels,
                &BandSpec {
                    round,
                    padding_inner: 0.2,
                    padding_outer: 0.1,
                    ..BandSpec::default()
                },
                range,
            )
            .unwrap();
            let points = PointScale::resolve_d3(
                &labels,
                &PointSpec {
                    round,
                    ..PointSpec::default()
                },
                range,
            )
            .unwrap();
            for label in &labels {
                let center = bands.center(label).unwrap().unwrap();
                assert_eq!(bands.category_at(center).unwrap(), Some(label.as_str()));
                let p = points.center(label).unwrap().unwrap();
                assert_eq!(points.category_at(p).unwrap(), Some(label.as_str()));
            }
            let visible = bands.clone().with_window("b", "c").unwrap();
            assert_eq!(visible.domain(), labels);
            assert_eq!(visible.visible_domain(), ["b", "c"]);
            assert!(visible.center("a").unwrap().is_none());
            for label in visible.visible_domain() {
                assert_eq!(
                    visible
                        .category_at(visible.center(label).unwrap().unwrap())
                        .unwrap(),
                    Some(label.as_str())
                );
            }
        }
    }
    let collapsed = PointScale::resolve_d3(
        &["a".into(), "b".into(), "c".into()],
        &PointSpec {
            round: true,
            ..PointSpec::default()
        },
        Bounds::new(0., 1.).unwrap(),
    )
    .unwrap();
    assert_eq!(collapsed.category_at(0.9).unwrap(), Some("a"));
    // Binary64 destination rounding collapses some adjacent centers at this origin.
    let huge = PointScale::resolve_d3(
        &["a".into(), "b".into(), "c".into(), "d".into()],
        &PointSpec::default(),
        Bounds::new(1e16, 1e16 + 4.).unwrap(),
    )
    .unwrap();
    let labels = huge.visible_domain();
    for p in [1e16, 1e16 + 2., 1e16 + 4.] {
        let expected = labels
            .iter()
            .min_by(|a, b| {
                let a = (huge.center(a).unwrap().unwrap() - p).abs();
                let b = (huge.center(b).unwrap().unwrap() - p).abs();
                a.total_cmp(&b)
            })
            .unwrap();
        assert_eq!(huge.category_at(p).unwrap(), Some(expected.as_str()));
    }
    let duplicate = CategoryScale::band(
        BandSpec {
            domain: Some(vec!["a", "a", "b"]),
            ..BandSpec::default()
        },
        range,
    )
    .unwrap();
    assert_eq!(duplicate.domain(), ["a", "b"]);
    assert_eq!(duplicate.step(), 50.);
}
