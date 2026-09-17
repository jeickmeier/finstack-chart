//! GG18 combined model/facet/theme/math/extension authors, without a second engine.
use chart_core::{
    Revision,
    grammar::*,
    prelude::*,
    theme::{ElementTheme, ThemePreset},
    typography::MathFonts,
};
pub const CASES: usize = 4;
pub fn data() -> ChartResult<Data> {
    Data::columns()
        .column("x", (0..24).map(|i| (i / 2) as f64).collect::<Vec<_>>())
        .column(
            "y",
            (0..24)
                .map(|i| 2. + 0.3 * (i / 2) as f64 + ((i / 2) % 3) as f64 + (i % 2) as f64)
                .collect::<Vec<_>>(),
        )
        .column(
            "g",
            (0..24)
                .map(|i| if i % 2 == 0 { "a" } else { "b" })
                .collect::<Vec<_>>(),
        )
        .keys((0..24).map(|i| 9_007_199_254_740_993 + i))
        .build()
}
pub fn from_data(data: Data, mode: usize) -> ChartResult<Plot> {
    from_data_profile(data, mode, Profile::Ggplot2_4_0_3)
}
pub fn from_data_profile(data: Data, mode: usize, profile: Profile) -> ChartResult<Plot> {
    let method = if mode == 3 {
        ModelMethod::Registered {
            operation: OperationRef::new(
                chart_extension_example::models::PRESCRIBED_SLOPE,
                Revision::new(1),
            ),
            parameters: serde_json::json!({"slope":0.3,"envelope":0.5}),
        }
    } else {
        ModelMethod::Linear
    };
    let model = smooth().stat(
        model_stat(ModelOptions {
            method,
            n: 25,
            ..Default::default()
        })
        .x("x")
        .y("y")
        .group("g"),
    );
    let model = if mode == 1 {
        model.filter(filter("x").minimum(4.))
    } else {
        model
    };
    let p = plot(data)
        .extensions(chart_extension_example::registry()?)
        .profile(profile)
        .aes(aes().x("x").y("y").color("g").color_scale("groups"))
        .scale(color_discrete("groups").domain(["a", "b"]))
        .layer(points().key_glyph(
            chart_extension_example::key_glyphs::DIAMOND,
            Revision::new(1),
            serde_json::json!({"padding":0.1}),
        ))
        .layer(model)
        .theme(theme().elements(ElementTheme::preset(ThemePreset::Bw)?)?)
        .title(title(if mode == 3 {
            "Geographic model"
        } else {
            "Combined grammar"
        }))
        .subtitle(subtitle("").rich(math_text("frac(1,2)+sqrt(4)", MathFonts::default())?))
        .tag(rich_text(if mode == 3 { "B" } else { "A" }))
        .legend(legend().scale("groups").registered(
            chart_extension_example::guide_drawing::STRIP,
            Revision::new(1),
            serde_json::json!({}),
        ))
        .facet(facet_wrap("g").registered(
            chart_extension_example::facet_planner::REVERSE,
            Revision::new(1),
            serde_json::json!({"columns":2}),
        ));
    let p = if mode == 3 {
        p.coordinate(CoordinateSpec::Geographic(GeographicCoordinate {
            projection: GeoProjectionSelection::Crs(GeoCrs::WebMercator),
            default_crs: Some(GeoCrs::Wgs84),
            ..Default::default()
        }))
    } else {
        p.registered_coordinate(
            chart_extension_example::coordinates::WAVE,
            Revision::new(1),
            serde_json::json!({"amplitude":0.06}),
        )
    };
    let p = if mode == 2 {
        p.coordinate(CoordinateSpec::Cartesian(CartesianCoordinate {
            xlim: Some([
                Some(chart_core::composition::ScaleValue::Number(4.)),
                Some(chart_core::composition::ScaleValue::Number(11.)),
            ]),
            ..Default::default()
        }))
    } else {
        p
    };
    p.build()
}
pub fn author(mode: usize) -> ChartResult<Plot> {
    from_data(data()?, mode)
}
