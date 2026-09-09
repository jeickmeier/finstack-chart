//! CLR-04 / COL-05: publication retains floating inputs and lowers shared scene bytes.
use chart_core::{
    color::{self, ColorValue},
    prelude::*,
    scene::Primitive,
};
use chart_export::*;
use serde_json::json;
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
fn plot_value(value: ColorValue) -> Plot {
    plot(
        Data::columns()
            .column("x", [0., 1.])
            .column("y", [1., 2.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points().color(value))
    .build()
    .unwrap()
}
#[test]
fn retained_request_metadata_and_publication_keep_authored_space_after_disposal() {
    let value = ColorValue::from(color::hcl(390., 55., 60.)).with_opacity(0.375);
    let background = color::lab(97., -1., -3.);
    let (request, wire) = {
        let plot = plot_value(value);
        let wire = plot.to_json().unwrap();
        let output = Output::new(FONT).unwrap();
        (
            output
                .request(
                    &plot,
                    export_options(PageSize::points(300., 220.).unwrap()).background(background),
                )
                .unwrap(),
            wire,
        )
    };
    assert_eq!(
        request.profile().background.value(),
        ColorValue::from(background)
    );
    let manifest = request.manifest().unwrap();
    assert_eq!(
        manifest["definition"]["layers"][0]["style"]["color"]["value"]["space"],
        "Hcl"
    );
    assert_eq!(manifest["profile"]["background"]["value"]["space"], "Lab");
    let a = request.prepare().unwrap();
    let b = request.prepare().unwrap();
    let paints: Vec<_> = a
        .scene()
        .items()
        .iter()
        .filter_map(|i| match i.primitive {
            Primitive::Point { fill, .. } if i.layer.is_some() => Some(fill),
            _ => None,
        })
        .collect();
    assert_eq!(paints, vec![value.to_paint(); 2]);
    assert_eq!(a.scene_json().unwrap(), b.scene_json().unwrap());
    for format in [Format::Svg, Format::Pdf, Format::Png] {
        assert_eq!(
            a.export(format).unwrap().bytes,
            b.export(format).unwrap().bytes
        );
    }
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
}
#[test]
fn portable_profile_and_live_override_versions_reject_hidden_floating_capabilities() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/composition/portable-cases.json"
    ))
    .unwrap();
    let case = &cases[0];
    let mut profile: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/composition/profile.json")).unwrap();
    let value = serde_json::to_value(Paint::from(color::hsl(210., 0.5, 0.4))).unwrap();
    profile["output_theme"] = json!({"mark":value});
    let build = |p: &serde_json::Value| {
        chart_export::portable::PortableChart::new(
            &case["chart"].to_string(),
            &case["data"].to_string(),
            &p.to_string(),
            FONT.to_vec(),
        )
    };
    assert!(build(&profile).is_err());
    profile["version"] = 2.into();
    let mut chart = build(&profile).unwrap();
    assert!(!chart.export("svg").unwrap().is_empty());
    let mut begin = json!({"version":1,"operation":{"Begin":{"format":"svg","basis":"Current","output_theme":{"mark":value}}}});
    assert!(chart.export_control(&begin.to_string()).is_err());
    begin["version"] = 2.into();
    assert!(chart.export_control(&begin.to_string()).is_ok());
    profile["output_theme"] = json!({});
    profile["background"] = value;
    assert!(build(&profile).is_ok());
    profile["version"] = 1.into();
    assert!(build(&profile).is_err());
}
