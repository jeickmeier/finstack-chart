//! GG12 source-backed facet authors shared by independent Rust publication/native wrappers.
use chart_core::{grammar::*, prelude::*};
pub const CASES: usize = 17;
fn levels(v: &[&str]) -> Option<Vec<GroupValue>> {
    Some(v.iter().map(|s| GroupValue::Text((*s).into())).collect())
}
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = Data::columns()
        .column("x", [1., 2., 1., 4., 1., 10., 2.])
        .column("y", [1., 3., 2., 8., 10., 100., 4.])
        .column(
            "r",
            [
                Some("B"),
                Some("B"),
                Some("A"),
                Some("A"),
                Some("B"),
                Some("B"),
                None,
            ],
        )
        .column("nested", ["u", "u", "v", "v", "w", "w", "u"])
        .column("c", ["L", "L", "R", "R", "R", "R", "L"])
        .column("z", ["z1", "z2", "z1", "z2", "z1", "z2", "z1"])
        .build()?;
    let mut b = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points());
    let mut policy = FacetPolicy::default();
    let mut facet = facet_wrap("r").fields(["r", "c"]).columns(2);
    match mode {
        1 => {
            policy.drop = false;
            policy.levels = vec![levels(&["B", "A", "unused"]), levels(&["R", "L", "unused"])];
        }
        2 => {
            facet = facet_grid("r", "c").fields(["r", "nested", "c", "z"]);
            policy.row_fields = 2;
        }
        3 => {
            facet = facet_grid("r", "c").fields(["r", "nested", "c"]);
            policy.row_fields = 2;
            policy.margins = vec![0, 1, 2];
        }
        4 => {
            policy.shrink = false;
            facet = facet_wrap("c").free_y(true);
            b = b.layer(
                points()
                    .stat(
                        summary()
                            .x(1.)
                            .y("y")
                            .summary_helper(SummaryHelper::default()),
                    )
                    .color(rgb(220, 40, 40)),
            );
        }
        5 => {
            facet = facet_grid("r", "c").free_x(true).free_y(true);
            policy.space = FacetSpace::Free;
        }
        6 => policy.direction = FacetDirection::Tr,
        7 => policy.direction = FacetDirection::Bl,
        8..=11 => {
            policy.strip_position = [
                FacetStripPosition::Top,
                FacetStripPosition::Bottom,
                FacetStripPosition::Left,
                FacetStripPosition::Right,
            ][mode - 8]
        }
        12 => {
            policy.axes = FacetAxes::All;
            policy.axis_labels = FacetAxes::Margins;
        }
        13 => {
            policy.labeller.variable_names = true;
            policy.labeller.wrap_width = Some(12);
            policy.labeller.lookup.insert(
                "r".into(),
                [("B".into(), "Business group B".into())]
                    .into_iter()
                    .collect(),
            );
        }
        14 => {
            facet = facet_grid("r", "c");
            let annotation = Data::columns()
                .name("annotation")
                .column("r", ["C"])
                .column("y", [9.])
                .column("x", [9.])
                .build()?;
            b = b.layer(points().data(annotation).color(rgb(220, 40, 40)));
        }
        15 => {
            facet = facet_grid("r", "c");
            policy.as_table = false;
            policy.switch = FacetSwitch::Both;
        }
        16 => {
            facet = facet_grid("r", "c").fields(["c"]).free_x(true);
            policy.row_fields = 0;
            policy.space = FacetSpace::FreeX;
        }
        _ => {}
    }
    b.facet(facet.reference(policy)).build()
}
