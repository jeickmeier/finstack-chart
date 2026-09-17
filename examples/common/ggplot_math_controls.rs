//! GG14 mathematical text uses the same supplied faces in every consumer.
use chart_core::{
    composition::ScaleValue,
    grammar::{Profile, TextGeom, TextSizeUnit},
    prelude::*,
    services::ResourceDescriptor,
    typography::MathFonts,
};
pub const CASES: usize = 9;
pub fn author(
    mode: usize,
    fonts: MathFonts,
    fallback: ResourceDescriptor,
) -> chart_core::ChartResult<Plot> {
    let labels: [&str; 4] = match mode {
        0 => [
            "x[i]^2",
            "frac(alpha,beta)",
            "sqrt(x,3)",
            "sum(x[i],i==1,n)",
        ],
        1 => ["hat(x)", "widehat(x+y)", "widetilde(x+y)", "ring(x)"],
        2 => [
            "bgroup(\"(\",atop(x,y),\")\")",
            "integral(f(x)*dx,a,b)",
            "bold(x)+italic(y)",
            "phantom(x)*y",
        ],
        3 => [
            "alpha %in% A",
            "x %->% y",
            "scriptstyle(x[i])",
            "group(langle,x,rangle)",
        ],
        8 => [
            "paste(\"café\",frac(alpha,beta))",
            "paste(\"κόσμος\",sqrt(x))",
            "paste(\"мир\",x[i]^2)",
            "paste(\"naïve\",hat(y))",
        ],
        _ => [
            "plain(P)(X==x)",
            "frac(1,sqrt(2*pi))*e^{-x^2/2}",
            "bolditalic(x)",
            "list(alpha,beta,gamma)",
        ],
    };
    let data = Data::columns()
        .column("x", [1., 3., 1., 3.])
        .column("y", [3., 3., 1., 1.])
        .column("label", labels)
        .column("group", categorical(["alpha", "beta", "alpha", "beta"]))
        .build()?;
    let style = text_style().fallback(fallback);
    let headline = math_text(
        "paste(plain(Plotmath),\": \",frac(alpha^2,beta))",
        fonts.clone(),
    )?
    .style(style.clone());
    let ticks = text_style().math(fonts.clone())?.fallback(fallback);
    let mut p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            points()
                .text_geom(TextGeom {
                    math: Some(fonts.clone()),
                    units: TextSizeUnit::Points,
                    size: 16.,
                    angle: if mode == 8 { 30. } else { 0. },
                    ..Default::default()
                })
                .text_label("label"),
        )
        .x_axis(
            x_axis()
                .scale(scale_linear().domain(0., 4.))
                .ticks([
                    (ScaleValue::Number(1.), "alpha".into()),
                    (ScaleValue::Number(3.), "frac(1,2)".into()),
                ])
                .text_style(ticks)
                .rich_label(math_text("sum(x[i],i==1,n)", fonts.clone())?.style(style.clone())),
        )
        .y_axis(
            y_axis()
                .scale(scale_linear().domain(0., 4.))
                .rich_label(math_text("sqrt(y)", fonts.clone())?.style(style.clone())),
        )
        .title(title("").rich(headline));
    if mode == 5 {
        p = p.facet(
            facet_wrap("group").reference(chart_core::grammar::FacetPolicy {
                labeller: chart_core::grammar::FacetLabeller {
                    math: Some(fonts.clone()),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    } else if mode == 6 {
        p = p.layer(points().aes(aes().color("group").y(0.5))).legend(
            legend()
                .aesthetic(chart_core::grammar::LegendAesthetic::Color)
                .options(chart_core::grammar::LegendOptions {
                    math: Some(fonts.clone()),
                    ..Default::default()
                }),
        );
    } else if mode == 7 {
        p = p.layer(
            chart_core::prelude::labels()
                .panel_at(None, 0.5, 0.5)
                .rich(math_text("frac(alpha,beta)", fonts)?.style(style)),
        );
    }
    p.build()
}
