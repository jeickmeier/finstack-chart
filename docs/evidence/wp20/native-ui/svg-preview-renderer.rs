fn main() {
 let a: Vec<String> = std::env::args().collect();
 let mut o = usvg::Options::default();
 o.fontdb_mut().load_font_data(std::fs::read("fixtures/capability/fonts/NotoSans-Regular.ttf").unwrap());
 let svg = std::fs::read_to_string(&a[1]).unwrap().replace("ChartFont-1-1", "Noto Sans");
 let tree = usvg::Tree::from_data(svg.as_bytes(), &o).unwrap();
 let size = tree.size().to_int_size();
 let mut p = resvg::tiny_skia::Pixmap::new(size.width(),size.height()).unwrap();
 resvg::render(&tree,resvg::tiny_skia::Transform::identity(),&mut p.as_mut());
 p.save_png(&a[2]).unwrap();
}
