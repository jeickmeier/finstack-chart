//! Shared radial/link gallery: signed radii, gaps, annuli, tangents and edge identities.
use chart_core::plot::{
    shape_area_radial, shape_line_radial, shape_link, shape_link_horizontal, shape_link_radial,
    shape_link_vertical,
};
use chart_core::{
    ChartResult, grammar::NumericAesthetic as A, prelude::*, shape::CurveSpec, theme::NamedTheme,
};
pub fn figure(preset: NamedTheme) -> ChartResult<Plot> {
    let labels = [
        "Linear spiral",
        "Closed basis",
        "Radial area",
        "Smooth annulus",
        "Signed radii",
        "Defined gaps",
        "Horizontal links",
        "Vertical links",
        "Step links",
        "Radial links",
    ];
    let mut x = vec![];
    let mut y = vec![];
    let mut x2 = vec![];
    let mut y2 = vec![];
    let mut angle = vec![];
    let mut radius = vec![];
    let mut inner = vec![];
    let mut end = vec![];
    let mut outer = vec![];
    let mut groups = vec![];
    let mut panels = vec![];
    let mut slots = vec![];
    let mut keys = vec![];
    for (slot, label) in labels.iter().enumerate() {
        let n = if slot >= 6 {
            3
        } else if slot == 4 {
            4
        } else {
            13
        };
        for i in 0..n {
            let cartesian = (6..=8).contains(&slot);
            x.push(if cartesian { 0.5 } else { 2. });
            y.push(if cartesian { 0.75 + i as f64 * 0.5 } else { 2. });
            x2.push(3.5);
            y2.push(3.25 - i as f64 * 0.5);
            let a = i as f64 * std::f64::consts::PI / 6.;
            angle.push(if slot == 5 && i == 6 { None } else { Some(a) });
            radius.push(if slot == 4 {
                [24., -24., 32., -12.][i]
            } else {
                20. + i as f64
            });
            inner.push(8. + (i % 3) as f64);
            end.push(a + 1.5);
            outer.push(32.);
            groups.push(if slot >= 6 {
                ['A', 'B', 'C'][i].to_string()
            } else {
                "A".into()
            });
            panels.push((*label).to_string());
            slots.push(slot as i64);
            keys.push(9007199254741001 + (slot * 100 + i) as u64);
        }
    }
    let data = Data::columns()
        .name("radial-links")
        .column("x", x)
        .column("y", y)
        .column("x2", x2)
        .column("y2", y2)
        .column("angle", angle)
        .column("radius", radius)
        .column("inner", inner)
        .column("end", end)
        .column("outer", outer)
        .column("group", categorical(groups))
        .column("panel", categorical(panels))
        .column("slot", slots)
        .keys(keys)
        .build()?;
    let mut draft = plot(data.clone())
        .aes(
            aes()
                .x("x")
                .y("y")
                .x2("x2")
                .y2("y2")
                .group("group")
                .color("group"),
        )
        .theme(theme().preset(preset))
        .title(title(format!("Radial shapes and links / {preset:?}")))
        .subtitle(subtitle(
            "Clockwise angles, signed radii, gaps and source edges",
        ))
        .facet(facet_wrap("panel").columns(2).gap(12.))
        .x_axis(x_axis().scale(scale_linear().domain(0., 4.)).visible(false))
        .y_axis(y_axis().scale(scale_linear().domain(0., 4.)).visible(false))
        .scale(color_discrete("group").domain(["A", "B", "C"]))
        .legend(legend().scale("group").title("Source"));
    for (slot, label) in labels.into_iter().enumerate() {
        let mut layer = match slot {
            0 => shape_line_radial(),
            1 => shape_line_radial().curve(CurveSpec::BasisClosed),
            2 => shape_area_radial(),
            3 => shape_area_radial().curve(CurveSpec::Basis),
            4 => shape_line_radial().curve(CurveSpec::Cardinal { tension: 0.2 }),
            5 => shape_line_radial().curve(CurveSpec::Step),
            6 => shape_link_horizontal(),
            7 => shape_link_vertical(),
            8 => shape_link(CurveSpec::Step),
            _ => shape_link_radial(),
        };
        if slot < 6 {
            layer = layer.shape_value(A::Angle, data.field("angle")?);
            layer = if slot == 2 || slot == 3 {
                layer
                    .shape_value(A::InnerRadius, data.field("inner")?)
                    .shape_value(A::OuterRadius, data.field("radius")?)
            } else {
                layer.shape_value(A::Radius, data.field("radius")?)
            };
        }
        if slot == 9 {
            layer = layer
                .shape_value(A::StartAngle, data.field("angle")?)
                .shape_value(A::EndAngle, data.field("end")?)
                .shape_value(A::InnerRadius, data.field("inner")?)
                .shape_value(A::OuterRadius, data.field("outer")?);
        }
        draft = draft.layer(
            layer
                .name(label)
                .filter(filter("slot").minimum(slot as f64).maximum(slot as f64)),
        );
    }
    draft.build()
}
