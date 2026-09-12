//! FIX-P05/06 independent destination error, transforms, budgets and version boundaries.
use chart_core::{
    path::{Affine, Command, PathGeometry, PathRequest},
    prelude::*,
    scene::PathCommand,
};
use std::f64::consts::FRAC_PI_2;

fn xy(point: chart_core::Point) -> [f64; 2] {
    [point.x(), point.y()]
}
fn cubic(p: [[f64; 2]; 4], t: f64) -> [f64; 2] {
    let u = 1. - t;
    std::array::from_fn(|i| {
        u * u * u * p[0][i]
            + 3. * u * u * t * p[1][i]
            + 3. * u * t * t * p[2][i]
            + t * t * t * p[3][i]
    })
}
#[test]
fn circular_lowering_obeys_coordinate_bound_at_multiple_scales_and_directions() {
    for radius in [1e-8, 1., 1e6] {
        for reverse in [false, true] {
            let sign = if reverse { -1. } else { 1. };
            let mut p = path();
            p.arc([0., 0.], radius, 0., sign * FRAC_PI_2, reverse)
                .unwrap();
            for tolerance in [radius * 1e-2, radius * 1e-6] {
                let commands = p.geometry().lower(tolerance, 10000).unwrap();
                let count = commands.len() - 1;
                let mut first = [radius, 0.];
                for (index, command) in commands[1..].iter().enumerate() {
                    let PathCommand::CubicTo(a, b, c) = command else {
                        panic!("expected cubic arc")
                    };
                    for j in 0..=64 {
                        let t = j as f64 / 64.;
                        let actual = cubic([first, xy(*a), xy(*b), xy(*c)], t);
                        let angle = sign * FRAC_PI_2 * (index as f64 + t) / count as f64;
                        let error = (actual[0] - radius * angle.cos())
                            .hypot(actual[1] - radius * angle.sin());
                        assert!(
                            error <= tolerance,
                            "radius {radius}, direction {reverse}, error {error} > {tolerance}"
                        );
                    }
                    first = xy(*c);
                }
            }
        }
    }
}

#[test]
fn transforms_preserve_arcs_or_lower_with_destination_bounds_and_budget() {
    let mut p = path();
    p.arc([0., 0.], 10., 0., FRAC_PI_2, false).unwrap();
    let reflected = p
        .geometry()
        .transformed(Affine::new([-2., 0., 0., 2., 30., 40.]).unwrap(), 0.01, 100)
        .unwrap();
    assert!(matches!(
        reflected.commands()[1],
        Command::Arc {
            radius: 20.,
            clockwise: false,
            ..
        }
    ));
    assert_eq!(reflected.commands()[0], Command::MoveTo([10., 40.]));
    let ellipse = p
        .geometry()
        .transformed(
            Affine::new([2., 0., 1., 3., 30., 40.]).unwrap(),
            0.001,
            10000,
        )
        .unwrap();
    assert!(
        !ellipse
            .commands()
            .iter()
            .any(|c| matches!(c, Command::Arc { .. }))
    );
    let bounds = ellipse.bounds(0.001, 10000).unwrap().unwrap();
    assert_eq!(
        serde_json::from_str::<PathGeometry>(&serde_json::to_string(&ellipse).unwrap()).unwrap(),
        ellipse,
        "Numeric retained geometry must round-trip without changing its identity"
    );
    for j in 0..=1000 {
        let a = FRAC_PI_2 * j as f64 / 1000.;
        let x = 30. + 20. * a.cos() + 10. * a.sin();
        let y = 40. + 30. * a.sin();
        assert!(
            x >= bounds.origin().x()
                && x <= bounds.max_x()
                && y >= bounds.origin().y()
                && y <= bounds.max_y()
        );
    }
    assert_eq!(
        p.geometry().lower(1e-12, 2).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    let limited = Path::with_options(
        Precision::Unrounded,
        PathLimits {
            max_subdivisions: 1,
            ..Default::default()
        },
    )
    .unwrap();
    let mut limited = limited;
    limited.arc([0., 0.], 10., 0., FRAC_PI_2, false).unwrap();
    assert_eq!(
        limited.lower(0.01).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    for value in [f64::NAN, 0., -1., f64::INFINITY] {
        assert!(p.geometry().lower(value, 100).is_err());
    }
    let mut closed = path();
    closed.rect(0., 0., 10., 10.).unwrap();
    closed.close_path().unwrap();
    closed.line_to(20., 0.).unwrap();
    let commands = closed.lower(0.01).unwrap();
    assert!(matches!(
        commands[4..],
        [
            PathCommand::Close,
            PathCommand::Close,
            PathCommand::LineTo(_)
        ]
    ));
    assert!(
        PathGeometry::from_commands(vec![Command::CubicTo([0., 0., f64::NAN, 0., 1., 1.])], 10)
            .is_err()
    );
    assert!(
        serde_json::from_str::<PathGeometry>(
            r#"{"commands":[{"Arc":{"radius":-1,"large":false,"clockwise":true,"to":[1,2]}}]}"#
        )
        .is_err()
    );
}

#[test]
fn primary_path_snapshot_and_versions_preserve_legacy_contract() {
    let data = Data::columns()
        .column("x", [0., 1.])
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let legacy = plot(data.clone())
        .aes(aes().x("x").y("y"))
        .layer(points())
        .build()
        .unwrap();
    let old = legacy.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&old).unwrap()["version"],
        1
    );
    assert!(!old.contains("\"paths\""));
    let mut p = path();
    p.rect(0., 0., 10., 10.).unwrap();
    let annotation = vector_path("owned", p.geometry());
    p.line_to(50., 50.).unwrap();
    let current = legacy.edit().annotation(annotation).build().unwrap();
    let encoded = current.to_json().unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["version"], 2);
    assert_eq!(
        value["definition"]["figure"]["paths"][0]["geometry"]["commands"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        Plot::from_json(&encoded).unwrap().to_json().unwrap(),
        encoded
    );
    let mut downgraded = value.clone();
    downgraded["version"] = 1.into();
    assert!(Plot::from_json(&downgraded.to_string()).is_err());
    downgraded["definition"]["figure"]["version"] = 1.into();
    assert!(Plot::from_json(&downgraded.to_string()).is_err());
    assert_eq!(legacy.to_json().unwrap(), old);
    assert!(
        current
            .edit()
            .annotation(vector_path("owned", path().geometry()))
            .build()
            .is_ok()
    );
    let request = PathRequest {
        version: 1,
        digits: Some(3.),
        limits: Default::default(),
        operations: vec![PathOp::Rect([0., 0., 10., 10.])],
    };
    let request = serde_json::to_string(&request).unwrap();
    assert_eq!(
        Path::from_json(&request).unwrap().to_svg().unwrap(),
        "M0,0h10v10h-10Z"
    );
    assert!(Path::from_json(&request.replace("\"version\":1", "\"version\":2")).is_err());
}

#[test]
fn every_numeric_argument_rejects_nonfinite_without_changing_the_builder() {
    for (name, args) in [
        ("moveTo", 2),
        ("lineTo", 2),
        ("quadraticCurveTo", 4),
        ("bezierCurveTo", 6),
        ("arcTo", 5),
        ("arc", 5),
        ("rect", 4),
    ] {
        for index in 0..args {
            for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut p = path();
                p.move_to(0., 0.).unwrap();
                let before = p.clone();
                let mut values = vec![1.; args];
                values[index] = bad;
                assert_eq!(
                    p.draw(name, &values, false).unwrap_err().code,
                    DiagnosticCode::NumericalDomain
                );
                assert_eq!(p, before);
                p.line_to(2., 3.).unwrap();
                assert_eq!(p.current_point(), Some([2., 3.]));
            }
        }
    }
}

#[test]
fn replay_and_scene_snapshots_preserve_supplied_metadata_without_vertex_targets() {
    use chart_core::{path::PathSink, provenance::Target, scene::*, services::Units, *};
    struct Consumer {
        target: Target,
        commands: Vec<Command>,
    }
    impl PathSink for Consumer {
        fn command(&mut self, command: &Command) -> ChartResult<()> {
            self.commands.push(*command);
            Ok(())
        }
    }
    let target = Target::Derived {
        id: DerivedId::new(7),
        model: "supplied.circular.geometry".into(),
        model_version: Revision::new(2),
        inputs: vec![],
    };
    let mut p = path();
    p.arc([10., 10.], 5., 0., std::f64::consts::TAU, false)
        .unwrap();
    let mut consumer = Consumer {
        target: target.clone(),
        commands: vec![],
    };
    p.replay(&mut consumer).unwrap();
    assert_eq!(consumer.target, target);
    assert_eq!(consumer.commands.len(), 3); // One model target, not three source rows.
    let clip = Rect::new(0., 0., 12., 20.).unwrap();
    let mut items = vec![SceneItem {
        guide: None,
        layer: Some(LayerId::new(17)),
        clip: Some(clip),
        primitive: Primitive::VectorPath {
            dashes: vec![],
            geometry: PathGeometry::from_commands(consumer.commands, 10).unwrap(),
            fill: Some(Color {
                red: 30,
                green: 40,
                blue: 50,
                alpha: 255,
            }),
            stroke: None,
        },
    }];
    let stamp = SceneStamp {
        definition: Revision::new(11),
        ..Default::default()
    };
    let scene = Scene::new(stamp, Units::Points, clip, &items, &[], Limits::default()).unwrap();
    p.rect(50., 50., 10., 10.).unwrap();
    items.clear();
    assert_eq!(scene.stamp(), stamp);
    assert_eq!(scene.items()[0].layer, Some(LayerId::new(17)));
    assert_eq!(scene.items()[0].clip, Some(clip));
    let Primitive::VectorPath { geometry, .. } = &scene.items()[0].primitive else {
        panic!("retained path")
    };
    assert_eq!(geometry.commands().len(), 3);
    assert_eq!(scene.wire_version(), 2);
}
