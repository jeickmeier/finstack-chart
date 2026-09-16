//! FIX-GG06 independently authored Rust bin values, replay and publications.
#[path = "../../../examples/common/ggplot_bin_stat_controls.rs"]
mod fixtures;
use chart_core::grammar::PreparedRows;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let mut records = vec![];
    let mut publications = 0;
    for (i, t) in fixtures::cases().iter().enumerate() {
        let p = fixtures::author(t)?;
        let wire = p.to_json()?;
        let q = chart_core::prelude::Plot::from_json(&wire)?;
        assert_eq!(q.to_json()?, wire);
        for (state, plot) in [("original", &p), ("replay", &q)] {
            let prepared = plot.chart()?.prepare()?;
            let PreparedRows::Binned(rows) = prepared.layers()[0].table().rows() else {
                panic!("bin rows")
            };
            let expected = t["result"].as_array().unwrap();
            assert_eq!(rows.len(), expected.len());
            let mut values = vec![];
            for (row, expected) in rows.iter().zip(expected) {
                let mut actual =
                    serde_json::to_value(row.statistics.as_ref().expect("reference statistics"))?;
                actual["xmin"] = row.start.into();
                actual["xmax"] = row.end.into();
                for (key, value) in actual.as_object().unwrap() {
                    if value.is_null() {
                        assert!(expected[key].is_null());
                    } else {
                        let (a, b) = (value.as_f64().unwrap(), expected[key].as_f64().unwrap());
                        assert!(
                            (a - b).abs() <= 3e-12 * b.abs().max(1.),
                            "case {i} {key}: {a} != {b}"
                        );
                    }
                }
                values.push(actual);
            }
            records.push(serde_json::json!({"case":i,"state":state,"rows":values}));
        }
        if fixtures::selected(t) {
            let request = output.request(&q, export_options(PageSize::points(480., 320.)?))?;
            let frame = request.prepare()?;
            for (format, suffix) in [
                (Format::Svg, "svg"),
                (Format::Pdf, "pdf"),
                (Format::Png, "png"),
            ] {
                std::fs::write(
                    out.join(format!("bin-{i}.{suffix}")),
                    frame.export(format)?.bytes,
                )?;
                publications += 1;
            }
            println!("{}", fixtures::name(t));
        }
    }
    assert_eq!(records.len(), 200);
    assert_eq!(publications, 24);
    std::fs::write(out.join("records.json"), serde_json::to_vec(&records)?)?;
    println!("PASS Rust GG06 bins: 200 states, 24 publications.");
    Ok(())
}
