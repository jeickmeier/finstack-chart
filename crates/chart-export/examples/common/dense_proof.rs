use chart_export::portable::PortableChart;
use serde_json::{Value, json};
use std::{fs, path::Path};
pub fn run(root: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<Value> =
        serde_json::from_str(&fs::read_to_string(root.join("fixtures/dense/cases.json"))?)?;
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let mut chart = PortableChart::new(
            &case["chart"].to_string(),
            &case["data"].to_string(),
            &fs::read_to_string(root.join("fixtures/interaction/profile.json"))?,
            fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
        )?;
        let stamp = serde_json::from_str::<Value>(&chart.present()?)?["stamp"].clone();
        let before: Value = serde_json::from_str(&chart.semantics()?)?;
        let exact = chart.export("svg")?;
        let result: Value =
            serde_json::from_str(&chart.dense_preview(&case["request"].to_string())?)?;
        assert_eq!(before, serde_json::from_str::<Value>(&chart.semantics()?)?);
        assert_eq!(exact, chart.export("svg")?);
        let described: Value = serde_json::from_str(&chart.query(
            &json!({"scene":stamp,"query":{"Describe":{"offset":123,"limit":1}}}).to_string(),
        )?)?;
        fs::write(
            output.join(format!("dense-{name}.svg")),
            result["svg"].as_str().unwrap(),
        )?;
        let mut database = usvg::fontdb::Database::new();
        database.load_font_data(fs::read(
            root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"),
        )?);
        let font_id = database.faces().next().unwrap().id;
        let options = usvg::Options {
            dpi: 72.,
            fontdb: std::sync::Arc::new(database),
            font_resolver: usvg::FontResolver {
                select_font: Box::new(move |_, _| Some(font_id)),
                select_fallback: Box::new(|_, _, _| None),
            },
            ..Default::default()
        };
        let tree = usvg::Tree::from_str(result["svg"].as_str().unwrap(), &options)?;
        let size = tree.size().to_int_size();
        let mut pixels =
            resvg_export::tiny_skia::Pixmap::new(size.width() * 2, size.height() * 2).unwrap();
        resvg_export::render(
            &tree,
            resvg_export::tiny_skia::Transform::from_scale(2., 2.),
            &mut pixels.as_mut(),
        );
        pixels.save_png(output.join(format!("dense-{name}.png")))?;
        let mut trace = result.clone();
        trace.as_object_mut().unwrap().remove("svg");
        trace["described"] = described;
        trace["semantics"] = before;
        fs::write(
            output.join(format!("dense-{name}.json")),
            serde_json::to_vec_pretty(&trace)?,
        )?;
        chart.dispose();
    }
    Ok(())
}
