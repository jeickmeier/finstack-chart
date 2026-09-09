//! Shared external-style primary authoring examples over the checked-in source data.
//! These construct components explicitly; fixture chart definitions are never decoded.
use chart_core::{data::*, grammar::*, portable::BatchWire, prelude::*, scene::Color};
use serde_json::Value;

fn data(case: &Value) -> Data {
    dataset(case, 0)
}
pub fn dataset(case: &Value, index: usize) -> Data {
    let batch: BatchWire =
        serde_json::from_value(case["data"]["datasets"][index]["batch"].clone()).unwrap();
    let mut builder = Data::columns()
        .identity(
            case["data"]["datasets"][index]["id"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap(),
        )
        .name(format!("data_{index}"))
        .keys(batch.keys.iter().map(|k| k.get()))
        .schema_version(batch.schema_version);
    for (field, source) in batch.fields.iter().zip(batch.columns) {
        let mut value = match source.values {
            ColumnValues::Float64(v) => column(v),
            ColumnValues::Int64(v) => column(v),
            ColumnValues::UInt64(v) => column(v),
            ColumnValues::Boolean(v) => column(v),
            ColumnValues::Utf8(v) => column(v),
            ColumnValues::Categorical { codes, dictionary } => {
                categorical(codes.iter().map(|c| dictionary[*c as usize].clone()))
            }
            ColumnValues::Timestamp(v) => {
                let FieldKind::Timestamp(kind) = &field.kind else {
                    panic!("timestamp kind")
                };
                timestamps(v, kind.unit, kind.timezone.clone())
            }
        }
        .validity(source.validity)
        .nullable(field.nullable);
        if let Some(formatted) = source.formatted {
            value = value.formatted(formatted);
        }
        if let Some(unit) = &field.unit {
            value = value.unit(unit);
        }
        if let Some(label) = &field.label {
            value = value.label(label);
        }
        builder = builder.column(&field.name, value);
    }
    builder.build().unwrap()
}
fn color() -> Color {
    Color {
        red: 180,
        green: 200,
        blue: 215,
        alpha: 140,
    }
}
fn build_stat(name: &str, data: Data) -> Plot {
    let order = |n| (0..n).map(GroupValue::Int).collect();
    let layer =
        match name {
            "summary" | "transformed-summary" => {
                let stat = summary()
                    .x("x")
                    .quantiles(vec![0., 0.25, 0.5, 1.])
                    .empty_sum_zero(false);
                points()
                    .stat(if name == "transformed-summary" {
                        stat.transform(2., 10.)
                    } else {
                        stat
                    })
                    .after_stat(stat_aes().x(StatField::Group).y(StatField::Mean))
            }
            "count" => points()
                .stat(count().required(["x", "y"]).group("group"))
                .after_stat(stat_aes().x(StatField::Group).y(StatField::Count)),
            "ols" | "filtered-fit" => {
                let l = line()
                    .stat(fit().x("x").y("y"))
                    .after_stat(stat_aes().x(StatField::X).y(StatField::Y));
                if name == "filtered-fit" {
                    l.filter(filter("x").maximum(2.))
                } else {
                    l
                }
            }
            "auto-bin" => histogram().aes(aes().x("x")).bins(30),
            "overflow-bin" => rectangle()
                .stat(
                    bin()
                        .x("x")
                        .breaks(vec![0., 1., 2.])
                        .outliers(OutlierPolicy::Overflow),
                )
                .after_bin(
                    bin_aes()
                        .x(BinField::Start)
                        .x2(BinField::End)
                        .y(BinField::Count)
                        .y2(BinNumeric::Literal(0.)),
                ),
            "stack" | "normalize" => rectangle()
                .aes(aes().x(0.).x2(1.).y("y").y2(0.).group("group"))
                .position(stack(order(4)).normalize(name == "normalize")),
            "dodge" => rectangle()
                .aes(
                    aes()
                        .x("category")
                        .x2("category")
                        .y("y")
                        .y2(0.)
                        .group("group"),
                )
                .position(dodge(order(3)).width(0.9)),
            "jitter-data" | "jitter-display" => points()
                .aes(aes().x("x").y("y").group("group"))
                .position(jitter(u64::MAX).displacement(0.25, 0.5).units(
                    if name == "jitter-data" {
                        JitterUnits::Data
                    } else {
                        JitterUnits::Display
                    },
                )),
            _ => panic!("unimplemented fixture {name}"),
        };
    plot(data)
        .layer(layer.independent().color(color()))
        .build()
        .unwrap()
}
fn build_family(name: &str, data: Data) -> Plot {
    use chart_core::{layout::AxisSide, scales::*};
    let blue = Color {
        red: 30,
        green: 125,
        blue: 180,
        alpha: 210,
    };
    let red = Color {
        red: 220,
        green: 70,
        blue: 55,
        alpha: 210,
    };
    let missing = Color {
        red: 128,
        green: 128,
        blue: 128,
        alpha: 70,
    };
    let palette = vec![blue, red, missing];
    let mut p = plot(data).aes(aes().x("field_1").y("field_2"));
    let layer = match name {
        "family-log-gaps" => {
            p = p.y_axis(y_axis().scale(scale_log(10.)));
            line()
        }
        "family-symlog" => {
            p = p.y_axis(y_axis().scale(scale_symlog(2.)));
            points()
        }
        "family-point-color" => {
            p = p
                .x_axis(
                    x_axis().scale(
                        scale_point()
                            .categories(["Alpha", "Beta", "Gamma"])
                            .point_padding(0.5),
                    ),
                )
                .scale(
                    color_discrete("color")
                        .domain(["Alpha", "Beta", "Gamma"])
                        .palette(palette)
                        .missing(missing),
                );
            p = p.legend(legend().scale("color").generic_title());
            points().aes(aes().color("field_1").color_scale("color"))
        }
        "family-area" => area().baseline(0.),
        "family-ribbon" => ribbon().aes(aes().y2("field_3")),
        "family-heatmap" => {
            p = p.scale(
                color_continuous("color", 0., 5.)
                    .palette(vec![blue, red])
                    .clamp(true)
                    .missing(missing),
            );
            p = p.legend(legend().scale("color").generic_title());
            cells().aes(
                aes()
                    .x2("field_3")
                    .y2("field_4")
                    .color("field_5")
                    .color_scale("color"),
            )
        }
        "family-grouped-bars" | "family-stacked-bars" => {
            p = p
                .aes(aes().y("y").y2(0.).group("group"))
                .scale(color_discrete("color").palette(palette).missing(missing));
            let layer = if name == "family-grouped-bars" {
                rectangle()
                    .aes(aes().x("category").x2("category"))
                    .position(dodge((0..3).map(GroupValue::Int).collect()).width(0.9))
            } else {
                rectangle()
                    .aes(aes().x(0.).x2(1.))
                    .position(stack((0..4).map(GroupValue::Int).collect()))
            };
            return p
                .legend(legend().scale("color").generic_title())
                .layer(layer.color_group("color").color(color()))
                .build()
                .unwrap();
        }
        "family-ohlc-volume" => {
            p = p.y_axis(y_axis().name("volume").side(AxisSide::Right));
            p = p.layer(
                ohlc()
                    .width(12.)
                    .aes(aes().y2("field_3").low("field_4").high("field_5"))
                    .color(blue),
            );
            volume()
                .width(7.)
                .axes("x", "volume")
                .aes(aes().y("field_6"))
                .color(missing)
        }
        "family-secondary" => {
            p = p.y_axis(
                y_axis()
                    .name("fahrenheit")
                    .side(AxisSide::Right)
                    .secondary("y", 1.8, 32.),
            );
            points()
        }
        "family-session" => {
            p = p.x_axis(x_axis().scale(scale_session(SessionCalendar {
                id: "supplied-fixture-only".into(),
                revision: chart_core::Revision::new(1),
                unit: TimeUnit::Milliseconds,
                sessions: vec![
                    TimeBounds {
                        start: 1709164800000,
                        end: 1709251200000,
                    },
                    TimeBounds {
                        start: 1709510400000,
                        end: 1709596800000,
                    },
                ],
                closed: ClosedSessionPolicy::Omit,
            })));
            line()
        }
        "family-utc-leap" => {
            p = p.x_axis(x_axis().scale(scale_utc().interval(UtcInterval::Days(1))));
            line()
        }
        _ => panic!("unimplemented fixture {name}"),
    };
    if name == "family-session" || name == "family-utc-leap" {
        p = p.aes(
            aes()
                .x(Mapping::Timestamp {
                    field: "field_1".into(),
                    origin: 1709164800000,
                })
                .y("field_2"),
        );
    }
    p.layer(if name == "family-ohlc-volume" {
        layer
    } else {
        layer.color(blue)
    })
    .build()
    .unwrap()
}
/// Build the source-data corpus with normal typed authoring.
pub fn cases() -> Vec<(String, Plot)> {
    let mut result = Vec::new();
    for (fixture, make) in [
        (
            include_str!("../../fixtures/statistics/portable-cases.json"),
            build_stat as fn(&str, Data) -> Plot,
        ),
        (
            include_str!("../../fixtures/families/portable-cases.json"),
            build_family as fn(&str, Data) -> Plot,
        ),
    ] {
        let cases: Vec<Value> = serde_json::from_str(fixture).unwrap();
        assert_eq!(cases.len(), 12);
        for case in cases {
            let name = case["name"].as_str().unwrap();
            let built = make(name, data(&case));
            result.push((name.to_owned(), built));
        }
    }
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("../../fixtures/facets/portable-cases.json")).unwrap();
    assert_eq!(fixtures.len(), 7);
    for case in fixtures {
        let built = build_facet(&case);
        result.push((case["name"].as_str().unwrap().to_owned(), built));
    }
    result
}

fn panel(name: &str) -> PanelKey {
    PanelKey {
        values: vec![GroupValue::Text(name.into())],
    }
}
fn build_facet(case: &Value) -> Plot {
    let name = case["name"].as_str().unwrap();
    let blue = Color {
        red: 30,
        green: 125,
        blue: 180,
        alpha: 210,
    };
    let mut facet = facet_wrap("facet")
        .columns(2)
        .order(vec![panel("B"), panel("A")])
        .gap(12.)
        .collect_guides(true)
        .empty(EmptyPanels::Keep);
    if name == "facet-free-target" {
        facet = facet.free_y(true);
    }
    if name == "facet-grid-empty" {
        facet = facet_grid("facet", "group")
            .order(
                ["B", "A", "C"]
                    .into_iter()
                    .flat_map(|s| {
                        [2, 1].map(|g| PanelKey {
                            values: vec![GroupValue::Text(s.into()), GroupValue::Int(g)],
                        })
                    })
                    .collect(),
            )
            .gap(12.)
            .collect_guides(true)
            .empty(EmptyPanels::Keep);
    }
    let mut p = plot(data(case)).facet(facet);
    if name.ends_with("summary") {
        let scope = match name {
            "facet-group-summary" => StatScope::Group,
            "facet-panel-summary" => StatScope::Facet,
            _ => StatScope::Chart,
        };
        let mut layer = points()
            .stat(
                summary()
                    .x("y")
                    .group("group")
                    .quantiles(vec![0.5])
                    .empty_sum_zero(false),
            )
            .after_stat(stat_aes().x(StatField::Group).y(StatField::Mean))
            .scope(scope)
            .color(blue);
        if scope == StatScope::Chart {
            layer = layer.facet_target(FacetTarget::Broadcast);
        }
        return p.layer(layer).build().unwrap();
    }
    p = p
        .layer(
            points()
                .aes(
                    aes()
                        .x("x")
                        .y("y")
                        .group("group")
                        .color("group")
                        .color_scale("groups"),
                )
                .color(blue),
        )
        .scale(
            color_discrete("groups")
                .domain(["1", "2"])
                .palette(vec![
                    Color { alpha: 230, ..blue },
                    Color {
                        red: 218,
                        green: 105,
                        blue: 40,
                        alpha: 230,
                    },
                ])
                .missing(Color {
                    red: 120,
                    green: 120,
                    blue: 120,
                    alpha: 100,
                }),
        )
        .legend(legend().scale("groups").title("Group"));
    if name == "facet-shared-log" {
        p = p.y_axis(y_axis().scale(scale_log(10.)));
    }
    if case["data"]["datasets"].as_array().unwrap().len() > 1 {
        p = p.layer(
            rule()
                .data(dataset(case, 1))
                .aes(aes().x(0.).x2(2.).y("threshold").y2("threshold"))
                .facet_target(if name == "facet-free-target" {
                    FacetTarget::Panels(vec![panel("A")])
                } else {
                    FacetTarget::Broadcast
                })
                .color(Color {
                    red: 140,
                    green: 70,
                    blue: 90,
                    alpha: 220,
                }),
        );
    }
    p.build().unwrap()
}

/// Complete published composition cases with the destination's actual supplied faces.
pub fn composition_cases(
    bold: chart_core::services::ResourceDescriptor,
    arabic: chart_core::services::ResourceDescriptor,
) -> Vec<(String, Plot)> {
    use chart_core::{
        composition::{Anchor, Collision, ScaleValue},
        scene::{GradientDirection, LinearGradient},
        theme::{NamedTheme, Symbol},
        typography::TextDirection,
    };
    let fixtures: Vec<Value> = serde_json::from_str(include_str!(
        "../../fixtures/composition/portable-cases.json"
    ))
    .unwrap();
    assert_eq!(fixtures.len(), 3);
    fixtures
        .into_iter()
        .map(|case| {
            let name = case["name"].as_str().unwrap();
            let terminal = name == "composition-terminal";
            let blue = Color {
                red: 30,
                green: 125,
                blue: 180,
                alpha: 210,
            };
            let trace = line().aes(aes().x("x").y("y")).color(blue);
            let threshold = rule()
                .data(dataset(&case, 1))
                .aes(aes().x(0.).x2(2.).y("threshold").y2("threshold"))
                .facet_target(FacetTarget::Broadcast)
                .color(Color {
                    red: 140,
                    green: 70,
                    blue: 90,
                    alpha: 220,
                });
            let observations = points()
                .aes(
                    aes()
                        .x("x")
                        .y("y")
                        .group("group")
                        .color("group")
                        .color_scale("groups"),
                )
                .color(blue);
            let trace_id = trace.handle().unwrap();
            let threshold_id = threshold.handle().unwrap();
            let observations_id = observations.handle().unwrap();
            let typography = |size| text_style().size(size);
            let strong = |size| typography(size).font(bold).weight(700);
            let rgb = chart_core::theme::rgb;
            let gradient = LinearGradient {
                direction: GradientDirection::Vertical,
                start: if terminal {
                    rgb(27, 40, 56)
                } else {
                    rgb(234, 243, 252)
                },
                end: if terminal {
                    rgb(18, 27, 37)
                } else {
                    rgb(255, 255, 255)
                },
            };
            let theme = theme()
                .preset(if terminal {
                    NamedTheme::Terminal
                } else if name == "composition-grayscale" {
                    NamedTheme::Grayscale
                } else {
                    NamedTheme::Editorial
                })
                .style(style().gradient(gradient))
                .layer(observations_id, style().symbol(Symbol::Diamond))
                .layer(threshold_id, style().dashes(vec![4., 2.]))
                .layer(
                    trace_id,
                    style()
                        .mark(if terminal {
                            rgb(117, 187, 242)
                        } else {
                            rgb(35, 93, 145)
                        })
                        .stroke_width(1.5),
                );
            let p = plot(data(&case))
                .layer(trace)
                .layer(threshold)
                .layer(observations)
                .scale(
                    color_discrete("groups")
                        .domain(["1", "2"])
                        .palette(vec![
                            Color { alpha: 230, ..blue },
                            Color {
                                red: 218,
                                green: 105,
                                blue: 40,
                                alpha: 230,
                            },
                        ])
                        .missing(Color {
                            red: 120,
                            green: 120,
                            blue: 120,
                            alpha: 100,
                        }),
                )
                .legend(legend().scale("groups").title("Group"))
                .facet(
                    facet_wrap("facet")
                        .columns(2)
                        .order(vec![panel("B"), panel("A")])
                        .gap(12.),
                )
                .x_axis(
                    x_axis()
                        .text_style(typography(0.7).tabular(true))
                        .rich_label(rich_text("Observation x").style(typography(0.75)))
                        .rotation(-35.)
                        .format(number_format().precision(1)),
                )
                .y_axis(
                    y_axis()
                        .text_style(typography(0.7).tabular(true))
                        .rich_label(
                            rich_text("Value (units)")
                                .style(typography(0.75))
                                .rotation(-90.),
                        )
                        .format(number_format().precision(0)),
                )
                .title(
                    title("").rich(
                        rich_text("Small multiples")
                            .style(strong(1.4))
                            .run(text_run("  ·  one prepared population").style(typography(0.9))),
                    ),
                )
                .subtitle(
                    subtitle("").rich(
                        rich_text("Explicit fonts and vector paint   ")
                            .style(typography(0.72))
                            .run(
                                text_run("مرحبا بالعالم").style(
                                    typography(0.8)
                                        .language("ar")
                                        .direction(TextDirection::RightToLeft)
                                        .fallback(arabic),
                                ),
                            ),
                    ),
                )
                .caption(
                    caption("Figure 1 · Shared scales; the inset reuses the same source rows.")
                        .style(typography(0.72)),
                )
                .source_note(
                    source_note("Source: deterministic fixture. No external market data.")
                        .style(typography(0.64)),
                )
                .footnote(
                    footnote("Notes: x/y are source units; dashed rule = supplied reference.")
                        .style(typography(0.64)),
                )
                .panel_letter(panel_letter("(a)").panel(panel("B")).style(strong(0.8)))
                .panel_letter(panel_letter("(b)").panel(panel("A")).style(strong(0.8)))
                .inset(
                    inset()
                        .id("detail-a")
                        .panel(panel("A"))
                        .rectangle(0.52, 0.5, 0.45, 0.45)
                        .layer(trace_id)
                        .layer(observations_id)
                        .x_view(0., 1.)
                        .y_view(0., 4.)
                        .guides(false),
                )
                .layer(
                    callout()
                        .label(
                            labels()
                                .id("peak")
                                .at(2., 9.)
                                .panel(panel("A"))
                                .text("Peak 9")
                                .style(typography(0.68))
                                .offset(-43., 15.)
                                .priority(10)
                                .collision(Collision::Keep),
                        )
                        .to(Anchor::Data {
                            panel: Some(panel("A")),
                            scales: ScaleBindings::default(),
                            x: ScaleValue::Number(2.),
                            y: ScaleValue::Number(9.),
                        }),
                )
                .layer(
                    labels()
                        .id("units")
                        .figure_at(0.73, 0.95)
                        .text("180 × 120 mm")
                        .style(typography(0.6))
                        .overflow(true),
                )
                .layer(
                    labels()
                        .id("lambda")
                        .output_at(430., 12.)
                        .text("λ = −0.25")
                        .style(typography(0.64))
                        .overflow(true),
                )
                .layer(
                    labels()
                        .id("direct-b")
                        .panel_at(Some(panel("B")), 0.08, 0.4)
                        .text("Panel B")
                        .style(typography(0.65))
                        .collision(Collision::ShiftThenHide)
                        .priority(1),
                )
                .theme(theme)
                .build()
                .unwrap();
            (name.to_owned(), p)
        })
        .collect()
}
