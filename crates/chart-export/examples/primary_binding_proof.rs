//! FIX-AUTH07: actual typed Rust fixture independently compared with Python and WASM authors.
#[path = "../../chart-core/tests/support/authoring_families.rs"]
mod authoring_families;
#[path = "../tests/support/authoring_replay.rs"]
mod authoring_replay;
#[path = "../../../examples/common/authoring_fixtures.rs"]
#[allow(dead_code)]
mod authors;
use chart_core::prelude::*;
use chart_core::{
    data::TimeUnit,
    grammar::{StatField, StatNumeric},
};
use chart_export::{host::Runtime, *};
use serde_json::{Value, json};
use std::path::Path;
fn data(x: Vec<f64>, y: Vec<f64>, keys: Vec<u64>) -> ChartResult<Data> {
    Data::columns()
        .column("x", x)
        .column("y", y)
        .keys(keys)
        .build()
}
fn write(dir: &Path, name: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(dir.join(format!("{name}.json")), value)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::env::args().nth(1).ok_or("Supply artifact directory")?;
    let dir = Path::new(&destination);
    std::fs::create_dir_all(dir)?;
    let source = data(vec![1., 2., 3.], vec![2., 4., 3.], vec![1001, 1002, 1003])?;
    let authored = plot(source)
        .aes(aes().x("x").y("y"))
        .layer(line().name("prices").size(1.5))
        .layer(points().name("observations").size(3.))
        .layer(labels().id("peak").at(2., 4.).text("Peak").offset(0., 14.))
        .title(title("Primary parity"))
        .x_axis(x_axis().label("Time"))
        .y_axis(y_axis().label("Value"))
        .build()?;
    let mut chart = Runtime::new(&authored)?;
    let mut output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let (actions, inputs) = authoring_replay::run(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        &output,
    );
    write(dir, "runtime-actions", &actions.to_string())?;
    write(dir, "runtime-input", &inputs.to_string())?;
    write(
        dir,
        "runtime-stream",
        &authoring_replay::run_stream(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
            &output,
        )
        .to_string(),
    )?;
    let options = chart_export::host::Options::new(400., 260., "pt")?.set("dpi", "[96]")?;
    let bold = output.register_font(
        include_bytes!("../../../fixtures/composition/fonts/NotoSans-Bold.ttf").as_slice(),
    )?;
    let arabic = output.register_font(
        include_bytes!("../../../fixtures/composition/fonts/NotoSansArabic-Regular.ttf").as_slice(),
    )?;
    let mut family_semantics = serde_json::Map::new();
    let mut family_scenes = serde_json::Map::new();
    for (name, plot) in authoring_families::cases()
        .into_iter()
        .chain(authoring_families::composition_cases(bold, arabic))
    {
        let mut runtime = Runtime::new(&plot)?;
        family_semantics.insert(name.clone(), serde_json::from_str(&runtime.semantics()?)?);
        let family_options = if name.starts_with("composition-") {
            chart_export::host::Options::new(180., 120., "mm")?.set("dpi", "[96]")?
        } else if name == "facet-grid-empty" {
            chart_export::host::Options::new(600., 540., "pt")?.set("dpi", "[96]")?
        } else {
            options.clone()
        };
        let frame = runtime.present(&output, &family_options)?;
        family_scenes.insert(name.clone(), serde_json::from_str(&frame.scene_json()?)?);
        std::fs::write(
            dir.join(format!("{name}.png")),
            frame.export(Format::Png)?.bytes,
        )?;
    }
    write(
        dir,
        "families",
        &Value::Object(family_semantics).to_string(),
    )?;
    write(
        dir,
        "family-scenes",
        &Value::Object(family_scenes).to_string(),
    )?;

    write(dir, "initial", &chart.semantics()?)?;
    let frame = chart.present(&output, &options)?;
    write(dir, "scene", &frame.scene_json()?)?;
    std::fs::write(dir.join("primary.png"), frame.export(Format::Png)?.bytes)?;
    std::fs::write(dir.join("primary.svg"), frame.export(Format::Svg)?.bytes)?;
    std::fs::write(dir.join("primary.pdf"), frame.export(Format::Pdf)?.bytes)?;
    let selected: Value = serde_json::from_str(&chart.named_query(
        "Series",
        r#"{"layer":"observations","panel":null,"limit":64}"#,
        false,
        None,
    )?)?;
    let event = chart.command(
        "select",
        &json!({"targets":selected["targets"],"change":"Replace"}).to_string(),
        Some(0),
    )?;
    write(dir, "action", &event)?;
    let zoom = chart.named_query("Navigate",r#"{"axes":["x","y"],"panel":null,"action":{"Zoom":{"anchor":[200,130],"factor":2}},"boundary":"ClampToDomain"}"#,false,None)?;
    write(dir, "zoom", &zoom)?;
    let presented = chart.request(&output, &options)?;
    let tx = chart
        .transaction()?
        .id("primary-append")
        .data(
            "append",
            "data".into(),
            &data(vec![4.], vec![8.], vec![1004])?,
        )?
        .build()?;
    write(dir, "transaction", &chart.commit(&tx)?)?;
    write(dir, "replay", &chart.commit(&tx)?)?;
    let edited = authored.edit().title(title("Edited primary")).build()?;
    assert!(chart.apply_plot(&edited, 0)?);
    write(dir, "final", &chart.semantics()?)?;
    let current = chart.request(&output, &options.set("basis", r#"["current"]"#)?)?;
    chart.dispose();
    drop(output);
    drop(authored);
    drop(edited);
    let old = String::from_utf8(presented.prepare()?.export(Format::Svg)?.bytes)?;
    let new = String::from_utf8(current.prepare()?.export(Format::Svg)?.bytes)?;
    assert!(old.contains("Primary parity") && !old.contains("Edited primary"));
    assert!(new.contains("Edited primary"));
    let exact = Data::columns()
        .column(
            "t",
            timestamps(vec![i64::MAX - 1, i64::MAX], TimeUnit::Nanoseconds, "UTC"),
        )
        .column("unsigned", [u64::MAX - 1, u64::MAX])
        .column("signed", [i64::MIN, i64::MAX])
        .column("nullable", [Some(1.25), None])
        .column("group", categorical(["b", "a"]))
        .keys([u64::MAX - 1, u64::MAX])
        .build()?;
    let exact = plot(exact)
        .aes(aes().x(1.).y("nullable"))
        .layer(points())
        .build()?;
    write(dir, "exact", &exact.to_json()?)?;
    let statistics = plot(data(
        vec![1., 2., 3., 4.],
        vec![2., 4., 6., 8.],
        vec![11, 12, 13, 14],
    )?)
    .aes(aes().x("x").y("y"))
    .layer(
        points()
            .name("summary")
            .stat(summary().x("y"))
            .after_stat(stat_aes().x(StatNumeric::Literal(1.)).y(StatField::Mean)),
    )
    .layer(
        line()
            .name("fit")
            .stat(fit())
            .after_stat(stat_aes().x(StatField::X).y(StatField::Y)),
    )
    .build()?;
    write(dir, "statistics", &Runtime::new(&statistics)?.semantics()?)?;
    let custom_source = data(
        vec![0.25, 0.75, 1.25, 1.75],
        vec![1., 1., 1., 1.],
        vec![201, 202, 203, 204],
    )?;
    let custom = chart_extension_example::authoring::density_plot(
        custom_source.clone(),
        "x",
        vec![0., 1., 2.],
        false,
    )?;
    write(dir, "extension", &Runtime::new(&custom)?.semantics()?)?;
    let destination = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let extension_frame = destination
        .request(
            &custom,
            export_options(PageSize::points(400., 260.)?).dpi(96),
        )?
        .prepare()?;
    write(dir, "extension-scene", &extension_frame.scene_json()?)?;
    std::fs::write(
        dir.join("extension.png"),
        extension_frame.export(Format::Png)?.bytes,
    )?;
    let native = chart_extension_example::authoring::density_plot(
        custom_source,
        "x",
        vec![0., 1., 2.],
        true,
    )?;
    assert!(native.chart().is_ok());
    assert!(native.to_json().is_err());
    assert!(Runtime::new(&native).is_err());
    assert!(
        destination
            .request(&native, export_options(PageSize::points(400., 260.)?))
            .and_then(|r| r.prepare())
            .is_err()
    );
    println!(
        "PASS primary Rust source, actions, replay, definition-only edit and detached capture fixture"
    );
    Ok(())
}
