//! Independent GG15 geometry and projection contracts.
use chart_core::{DiagnosticCode, grammar::*};
fn resource(geometry: GeoGeometry) -> GeoFeatureCollection {
    GeoFeatureCollection {
        features: vec![GeoFeature {
            id: GroupValue::Text("region".into()),
            geometry,
            crs: None,
        }],
        crs: GeoCrs::Wgs84,
        axis_order: GeoAxisOrder::XY,
    }
}
#[test]
fn typed_geometry_retains_holes_multiparts_exact_ids_and_mixed_crs() {
    let outer = vec![[0., 0.], [6., 0.], [6., 6.], [0., 6.], [0., 0.]];
    let hole = vec![[2., 2.], [2., 4.], [4., 4.], [4., 2.], [2., 2.]];
    let mut input = resource(GeoGeometry::MultiPolygon(vec![
        vec![outer, hole],
        vec![vec![[10., 0.], [11., 0.], [11., 1.], [10., 0.]]],
    ]));
    input.features.push(GeoFeature {
        id: GroupValue::UInt(u64::MAX),
        geometry: GeoGeometry::Point([111319.49079327357, 0.]),
        crs: Some(GeoCrs::WebMercator),
    });
    input.validate(GeoLimits::default()).unwrap();
    let wire = serde_json::to_string(&input).unwrap();
    let replay: GeoFeatureCollection = serde_json::from_str(&wire).unwrap();
    assert_eq!(replay, input);
    assert!(wire.contains("18446744073709551615"));
}
#[test]
fn malformed_and_unbounded_features_fail_without_repair_or_io() {
    let bad = [
        GeoGeometry::LineString(vec![[0., 0.]]),
        GeoGeometry::Polygon(vec![vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]]),
        GeoGeometry::Point([f64::NAN, 0.]),
    ];
    for geometry in bad {
        assert_eq!(
            resource(geometry)
                .validate(GeoLimits::default())
                .unwrap_err()
                .code,
            DiagnosticCode::Validation
        );
    }
    let mut duplicate = resource(GeoGeometry::Empty);
    duplicate.features.push(duplicate.features[0].clone());
    assert!(duplicate.validate(GeoLimits::default()).is_err());
    let bounded = resource(GeoGeometry::MultiPoint(vec![[0., 0.], [1., 1.]]));
    assert_eq!(
        bounded
            .validate(GeoLimits {
                positions: 1,
                ..Default::default()
            })
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let recursive = resource(GeoGeometry::Collection(vec![GeoGeometry::Collection(
        vec![GeoGeometry::Empty],
    )]));
    assert!(
        recursive
            .validate(GeoLimits {
                depth: 1,
                ..Default::default()
            })
            .is_err()
    );
    for definition in [
        "+init=epsg:4326",
        "+proj=longlat +nadgrids=secret.gsb",
        "+proj=longlat +geoidgrids=remote.tif",
    ] {
        assert!(GeoCrs::Proj(definition.into()).validate().is_err());
    }
}
#[test]
fn mapproj_kernels_match_independent_pinned_oracle() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/mapproj-controls.json"
    ))
    .unwrap();
    let mut calls = 0;
    let isolated: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/mapproj-isolated-controls.json"
    ))
    .unwrap();
    for original_case in fixture["cases"].as_array().unwrap() {
        let case = if matches!(original_case["method"].as_str(), Some("tetra" | "hex")) {
            isolated["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| {
                    v["method"] == original_case["method"] && v["case"] == original_case["case"]
                })
                .unwrap()
        } else {
            original_case
        };
        let method: MapprojMethod = serde_json::from_value(case["method"].clone()).unwrap();
        if case["result"]["x"].is_null() {
            continue;
        }
        let params = match &case["parameters"] {
            serde_json::Value::Null => vec![],
            serde_json::Value::Array(a) => a.iter().map(|v| v.as_f64().unwrap()).collect(),
            v => vec![v.as_f64().unwrap()],
        };
        let o: [f64; 3] = serde_json::from_value(case["resolved"]["orientation"].clone()).unwrap();
        let spec = MapprojProjection::resolve(method, params, Some(o), [0., 0.]).unwrap();
        let lon = case["longitude"].as_array().unwrap();
        let lat = case["latitude"].as_array().unwrap();
        for i in 0..lon.len() {
            let actual = spec
                .project([
                    lon[i].as_f64().unwrap_or(f64::NAN),
                    lat[i].as_f64().unwrap_or(f64::NAN),
                ])
                .unwrap();
            for (j, name) in ["x", "y"].iter().enumerate() {
                if let Some(expected) = case["result"][name][i].as_f64() {
                    let value = actual
                        .unwrap_or_else(|| panic!("clipped {:?} {} {i}", method, case["case"]))[j];
                    // Elliptic ordinate and van der Grinten quadratic roots lose absolute precision at zero through square roots of cancellation residuals.
                    let tolerance =
                        if (method == MapprojMethod::Elliptic && j == 1 && expected.abs() < 1e-6)
                            || (method == MapprojMethod::Vandergrinten
                                && (expected.abs() < 1e-6
                                    || lat[i].as_f64().is_some_and(|v| v.abs() == 90.)))
                        {
                            8. * f64::EPSILON.sqrt()
                        } else {
                            2e-10 * (1. + expected.abs())
                        };
                    assert!(
                        (value - expected).abs() < tolerance,
                        "{:?} {} point{i} {name}: {value} != {expected}",
                        method,
                        case["case"]
                    );
                } else {
                    assert!(
                        actual.is_none() || !actual.unwrap()[j].is_finite(),
                        "{:?} {} point{i} {name} expected missing got {:?}",
                        method,
                        case["case"],
                        actual
                    );
                }
            }
        }
        calls += 1;
    }
    assert_eq!(calls, 318);
}
#[test]
fn feature_join_prepares_shared_polygon_holes_and_original_targets() {
    use chart_core::prelude::*;
    let collection = resource(GeoGeometry::Polygon(vec![
        vec![[0., 0.], [6., 0.], [6., 6.], [0., 6.], [0., 0.]],
        vec![[2., 2.], [2., 4.], [4., 4.], [4., 2.], [2., 2.]],
    ]));
    let data = Data::columns()
        .column("id", ["region", "unmatched"])
        .column("value", [1., 2.])
        .build()
        .unwrap();
    let p = plot(data)
        .layer(points().geography(collection, "id"))
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    assert_eq!(prepared.layers()[0].marks().len(), 1);
    let mark = &prepared.layers()[0].marks()[0];
    assert_eq!(mark.targets.len(), 1);
    let PreparedGeometry::Recipe(recipe) = &mark.geometry else {
        panic!("shared surface")
    };
    let PreparedRecipe::Surface(PreparedSurface::Polygon {
        contours,
        anchors,
        rule,
    }) = recipe.as_ref()
    else {
        panic!("polygon")
    };
    assert_eq!(contours.len(), 2);
    assert_eq!(anchors.len(), 1);
    assert_eq!(*rule, chart_core::scene::FillRule::EvenOdd);
}
#[test]
fn explicit_crs_axis_order_and_mercator_anchors() {
    let f = GeoTransform::new(&GeoCrs::Wgs84, &GeoCrs::WebMercator, GeoAxisOrder::XY).unwrap();
    let p = f.transform([1., 0.]).unwrap();
    assert!((p[0] - 111319.49079327357).abs() < 1e-8);
    assert!(p[1].abs() < 1e-8);
    let g = GeoTransform::new(&GeoCrs::WebMercator, &GeoCrs::Wgs84, GeoAxisOrder::XY).unwrap();
    let q = g.transform(p).unwrap();
    assert!((q[0] - 1.).abs() < 1e-12);
    assert!(q[1].abs() < 1e-12);
    let h = GeoTransform::new(&GeoCrs::Wgs84, &GeoCrs::WebMercator, GeoAxisOrder::YX).unwrap();
    assert_eq!(h.transform([0., 1.]).unwrap(), p);
    assert!(f.transform([0., 100.]).is_err());
}
#[test]
fn sf_polygon_centroid_and_interior_label_respect_holes_and_largest_part() {
    use chart_core::prelude::*;
    let ring = |x0, y0, x1, y1| vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]];
    let inputs = [
        (
            "donut",
            GeoGeometry::Polygon(vec![ring(0., 0., 6., 6.), ring(2., 2., 4., 4.)]),
        ),
        (
            "multipolygon",
            GeoGeometry::MultiPolygon(vec![
                vec![ring(0., 0., 1., 1.)],
                vec![ring(10., 0., 14., 4.)],
            ]),
        ),
        (
            "line",
            GeoGeometry::LineString(vec![[0., 0.], [2., 0.], [2., 6.]]),
        ),
        (
            "multipoint",
            GeoGeometry::MultiPoint(vec![[0., 0.], [2., 0.], [4., 6.]]),
        ),
    ];
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/geography-controls.json"
    ))
    .unwrap();
    for (name, geometry) in inputs {
        let expected = fixture["labels"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap();
        for (operation, field) in [
            (GeoOperation::Centroid, "centroid"),
            (GeoOperation::PointOnSurface, "point_on_surface"),
        ] {
            let data = Data::columns().column("id", ["region"]).build().unwrap();
            let p = plot(data)
                .layer(
                    points()
                        .geography(resource(geometry.clone()), "id")
                        .geography_operation(operation),
                )
                .build()
                .unwrap();
            let prepared = p.chart().unwrap().prepare().unwrap();
            let PreparedGeometry::Point(actual) = prepared.layers()[0].marks()[0].geometry else {
                panic!("label location")
            };
            for (j, v) in [actual.x(), actual.y()].into_iter().enumerate() {
                assert!(
                    (v - expected[field][j].as_f64().unwrap()).abs() < 1e-12,
                    "{name} {field} axis{j}: {v} expected {}",
                    expected[field]
                );
            }
        }
    }
}

#[test]
fn mapproj_descriptor_errors_match_pinned_arity_and_orientation_contract() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/mapproj-controls.json"
    ))
    .unwrap();
    let mut errors = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if case["result"]["message"].is_null() {
            continue;
        }
        let method: MapprojMethod = serde_json::from_value(case["method"].clone()).unwrap();
        let parameters = match &case["parameters"] {
            serde_json::Value::Null => vec![],
            serde_json::Value::Array(a) => a.iter().map(|v| v.as_f64().unwrap()).collect(),
            v => vec![v.as_f64().unwrap()],
        };
        let orientation: Result<[f64; 3], _> = serde_json::from_value(case["orientation"].clone());
        if let Ok(orientation) = orientation {
            assert!(
                MapprojProjection::resolve(method, parameters, Some(orientation), [0., 0.])
                    .is_err(),
                "{} {}",
                case["method"],
                case["case"]
            );
        } else {
            assert_eq!(case["case"], "invalid_orientation_length");
        }
        errors += 1;
    }
    assert_eq!(errors, 108);
}

#[test]
fn conformal_projection_reinitialization_is_immutable() {
    for method in [MapprojMethod::Tetra, MapprojMethod::Hex] {
        let first =
            MapprojProjection::resolve(method, vec![], Some([90., 0., 0.]), [0., 0.]).unwrap();
        let expected = first.project([30., 20.]).unwrap();
        for _ in 0..8 {
            let other = MapprojProjection::resolve(method, vec![], Some([30., 40., 15.]), [0., 0.])
                .unwrap();
            other.project([-50., -20.]).unwrap();
            assert_eq!(first.project([30., 20.]).unwrap(), expected);
            assert_eq!(
                MapprojProjection::resolve(method, vec![], Some([90., 0., 0.]), [0., 0.])
                    .unwrap()
                    .project([30., 20.])
                    .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn portable_crs_transforms_match_pinned_sf_proj_anchors() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/geography-controls.json"
    ))
    .unwrap();
    for case in fixture["crs"].as_array().unwrap() {
        let destination = match case["epsg"].as_u64().unwrap() {
            3857 => GeoCrs::WebMercator,
            4326 => GeoCrs::Wgs84,
            _ => GeoCrs::Proj(case["proj"].as_str().unwrap().into()),
        };
        let transform = GeoTransform::new(&GeoCrs::Wgs84, &destination, GeoAxisOrder::XY).unwrap();
        for (input, output) in case["input"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["output"].as_array().unwrap())
        {
            let input: [f64; 2] = serde_json::from_value(input.clone()).unwrap();
            let actual = transform.transform(input).unwrap();
            for i in 0..2 {
                let expected = output[i].as_f64().unwrap();
                assert!(
                    (actual[i] - expected).abs() < 1e-5,
                    "EPSG{} {input:?} axis{i}: {} expected{expected}",
                    case["epsg"],
                    actual[i]
                );
            }
        }
    }
}

#[test]
fn projected_feature_vertices_render_with_holes_replay_and_inspection() {
    use chart_core::{
        ChartResult, Rect, ResourceId, Revision,
        layout::{LayoutRequest, layout},
        prelude::*,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let polygon = GeoGeometry::Polygon(vec![
        vec![
            [-10., 20.],
            [20., 20.],
            [20., 60.],
            [-10., 60.],
            [-10., 20.],
        ],
        vec![[0., 30.], [10., 30.], [10., 40.], [0., 40.], [0., 30.]],
    ]);
    let author = plot(Data::columns().column("id", ["region"]).build().unwrap())
        .layer(points().geography(resource(polygon), "id"))
        .coordinate(CoordinateSpec::Geographic(GeographicCoordinate {
            projection: GeoProjectionSelection::Crs(GeoCrs::WebMercator),
            default_crs: Some(GeoCrs::Wgs84),
            graticule: GeoGraticule {
                datum: None,
                ..Default::default()
            },
            ..Default::default()
        }))
        .build()
        .unwrap();
    let replay = Plot::from_json(&author.to_json().unwrap()).unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 320., 240.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for axis in &mut request.axes {
        axis.visible = false;
    }
    let frame = std::sync::Arc::new(
        layout(
            replay.chart().unwrap().prepare().unwrap(),
            &request,
            &Metrics,
        )
        .unwrap(),
    );
    assert!(frame.scene().items().iter().any(|i| matches!(
        i.primitive,
        chart_core::scene::Primitive::ShapePath {
            fill_rule: chart_core::scene::FillRule::EvenOdd,
            ..
        }
    )));
    let mut contours: Vec<Vec<chart_core::Point>> = vec![];
    for item in frame.scene().items() {
        if let chart_core::scene::Primitive::ShapePath {
            geometry,
            fill: Some(_),
            ..
        } = &item.primitive
        {
            for command in geometry.lower(0.01, 10000).unwrap() {
                match command {
                    chart_core::scene::PathCommand::MoveTo(p) => contours.push(vec![p]),
                    chart_core::scene::PathCommand::LineTo(p) => {
                        contours.last_mut().unwrap().push(p)
                    }
                    _ => {}
                }
            }
            break;
        }
    }
    assert_eq!(contours.len(), 2);
    let midpoint = |points: &[chart_core::Point]| {
        let x = points.iter().map(|p| p.x()).fold(f64::INFINITY, f64::min) / 2.
            + points
                .iter()
                .map(|p| p.x())
                .fold(f64::NEG_INFINITY, f64::max)
                / 2.;
        let y = points.iter().map(|p| p.y()).fold(f64::INFINITY, f64::min) / 2.
            + points
                .iter()
                .map(|p| p.y())
                .fold(f64::NEG_INFINITY, f64::max)
                / 2.;
        chart_core::Point::new(x, y).unwrap()
    };
    let hole = midpoint(&contours[1]);
    let interior = chart_core::Point::new((contours[0][0].x() + hole.x()) / 2., hole.y()).unwrap();
    let inspector = chart_core::inspection::Inspector::new(frame, 0.1, 64).unwrap();
    assert_eq!(inspector.semantic_targets().len(), 1);
    assert!(
        inspector
            .query(hole, chart_core::inspection::InspectionMode::Auto)
            .hits
            .is_empty()
    );
    assert_eq!(
        inspector
            .query(interior, chart_core::inspection::InspectionMode::Auto)
            .hits
            .len(),
        1
    );
}

#[test]
fn sf_label_locations_use_destination_projection_before_inverse_to_overlay_crs() {
    use chart_core::prelude::*;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/geography-controls.json"
    ))
    .unwrap();
    for case in fixture["projected_labels"].as_array().unwrap() {
        let ring = serde_json::from_value(case["input"].clone()).unwrap();
        let operation = if case["operation"] == "centroid" {
            GeoOperation::Centroid
        } else {
            GeoOperation::PointOnSurface
        };
        let author = plot(Data::columns().column("id", ["region"]).build().unwrap())
            .layer(
                points()
                    .geography(resource(GeoGeometry::Polygon(vec![ring])), "id")
                    .geography_operation(operation),
            )
            .coordinate(CoordinateSpec::Geographic(GeographicCoordinate {
                projection: GeoProjectionSelection::Crs(GeoCrs::WebMercator),
                default_crs: Some(GeoCrs::Wgs84),
                ..Default::default()
            }))
            .build()
            .unwrap();
        let prepared = author.chart().unwrap().prepare().unwrap();
        assert_eq!(
            prepared.layers()[0].domains().x,
            Some(Extent {
                minimum: 0.,
                maximum: 10.
            })
        );
        assert_eq!(
            prepared.layers()[0].domains().y,
            Some(Extent {
                minimum: 0.,
                maximum: 60.
            })
        );
        let PreparedGeometry::Point(p) = prepared.layers()[0].marks()[0].geometry else {
            panic!("label anchor")
        };
        for (i, v) in [p.x(), p.y()].into_iter().enumerate() {
            assert!((v - case["output"][i].as_f64().unwrap()).abs() < 1e-12);
        }
    }
}

#[test]
fn sf_geometry_family_paint_defaults_and_explicit_overrides_are_independent() {
    use chart_core::prelude::*;
    let polygon = GeoGeometry::Polygon(vec![vec![[0., 0.], [2., 0.], [2., 2.], [0., 0.]]]);
    let data = || Data::columns().column("id", ["region"]).build().unwrap();
    let author = plot(data())
        .layer(points().geography(resource(polygon.clone()), "id"))
        .build()
        .unwrap();
    let prepared = author.chart().unwrap().prepare().unwrap();
    let style = prepared.layers()[0].marks()[0].style;
    assert_eq!(
        style.color,
        chart_core::scene::Color {
            red: 89,
            green: 89,
            blue: 89,
            alpha: 255
        }
    );
    assert_eq!(
        style.fill,
        Some(chart_core::scene::Color {
            red: 229,
            green: 229,
            blue: 229,
            alpha: 255
        })
    );
    assert!((style.stroke_width - 0.4267913385826772).abs() < 1e-12);
    let override_plot = plot(data())
        .layer(
            points()
                .geography(resource(polygon), "id")
                .color(chart_core::color::parse_r("red").unwrap())
                .linewidth(2.)
                .alpha(0.25),
        )
        .build()
        .unwrap();
    let prepared = override_plot.chart().unwrap().prepare().unwrap();
    let style = prepared.layers()[0].marks()[0].style;
    assert_eq!(
        style.color,
        chart_core::scene::Color {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 255
        }
    );
    assert_eq!(style.fill.unwrap().alpha, 64);
    assert!((style.stroke_width - 4.267913385826772).abs() < 1e-12);
}

#[test]
fn dateline_feature_and_hole_retain_pinned_sf_projected_vertices() {
    use chart_core::prelude::*;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/geography-controls.json"
    ))
    .unwrap();
    assert_eq!(fixture["dateline"]["valid"], "Valid Geometry");
    let rings: Vec<Vec<GeoPosition>> =
        serde_json::from_value(fixture["dateline"]["input"].clone()).unwrap();
    let author = plot(Data::columns().column("id", ["region"]).build().unwrap())
        .layer(points().geography(resource(GeoGeometry::Polygon(rings)), "id"))
        .coordinate(CoordinateSpec::Geographic(GeographicCoordinate {
            projection: GeoProjectionSelection::Crs(GeoCrs::WebMercator),
            ..Default::default()
        }))
        .build()
        .unwrap();
    let prepared = author.chart().unwrap().prepare().unwrap();
    let PreparedGeometry::Recipe(recipe) = &prepared.layers()[0].marks()[0].geometry else {
        panic!("polygon")
    };
    let PreparedRecipe::Surface(PreparedSurface::Polygon { contours, rule, .. }) = recipe.as_ref()
    else {
        panic!("polygon")
    };
    assert_eq!(*rule, chart_core::scene::FillRule::EvenOdd);
    for (i, ring) in contours.iter().enumerate() {
        for (j, p) in ring.iter().enumerate() {
            for (k, v) in [p.x(), p.y()].into_iter().enumerate() {
                assert!(
                    (v - fixture["dateline"]["projected"][i][j][k].as_f64().unwrap()).abs() < 1e-7
                );
            }
        }
    }
}
