//! HIR-07 shared source data and destination recipes for publication/native qualification.
use chart_core::{
    ChartResult,
    grammar::{HierarchyProjection, HierarchyRadius},
    hierarchy::{LayoutSpec, Tiler, TreeOptions, TreeSize, TreemapOptions},
    plot::Plot,
    prelude::*,
};
pub fn figures() -> ChartResult<Vec<(&'static str, Plot)>> {
    let data = Data::columns()
        .name("hierarchy")
        .keys((0..9).map(|i| 9_007_199_254_741_001 + i))
        .column(
            "id",
            ["root", "North", "South", "A", "B", "C", "D", "E", "F"],
        )
        .column(
            "parent",
            vec![
                None,
                Some("root"),
                Some("root"),
                Some("North"),
                Some("North"),
                Some("North"),
                Some("South"),
                Some("South"),
                Some("South"),
            ],
        )
        .column("value", [0., 0., 0., 8., 5., 3., 7., 4., 2.])
        .column(
            "group",
            [
                "root", "North", "South", "North", "North", "North", "South", "South", "South",
            ],
        )
        .build()?;
    let tree = || hierarchy_tree("id", "parent");
    let layers = [
        ("tree", tree()),
        (
            "cluster-horizontal",
            hierarchy_cluster("id", "parent").hierarchy_projection(HierarchyProjection::Horizontal),
        ),
        (
            "tree-radial",
            tree().hierarchy_projection(HierarchyProjection::Radial),
        ),
        (
            "cluster-radial",
            hierarchy_cluster("id", "parent").hierarchy_projection(HierarchyProjection::Radial),
        ),
        ("icicle", hierarchy_icicle("id", "parent")),
        (
            "sunburst",
            hierarchy_sunburst("id", "parent").hierarchy_projection(
                HierarchyProjection::Sunburst {
                    inner_radius: 18.,
                    radius: HierarchyRadius::Area,
                },
            ),
        ),
        (
            "treemap",
            hierarchy_treemap("id", "parent").hierarchy_layout(LayoutSpec::Treemap {
                options: TreemapOptions {
                    tile: Tiler::Resquarify(1.618_033_988_749_895),
                    padding_inner: 3.,
                    padding_top: 5.,
                    padding_right: 5.,
                    padding_bottom: 5.,
                    padding_left: 5.,
                    ..Default::default()
                },
                history: true,
                padding_sides: Default::default(),
                padding: None,
                tiler: None,
            }),
        ),
        ("pack", hierarchy_pack("id", "parent")),
        (
            "tree-node-size",
            tree().hierarchy_layout(LayoutSpec::Tree {
                options: TreeOptions {
                    mode: TreeSize::NodeSize([30., 80.]),
                    ..Default::default()
                },
                separation: None,
            }),
        ),
    ];
    layers
        .into_iter()
        .map(|(name, layer)| {
            Ok((
                name,
                plot(data.clone())
                    .layer(
                        layer
                            .name("hierarchy")
                            .hierarchy_value("value")
                            .hierarchy_label("id")
                            .aes(aes().color("id")),
                    )
                    .title(title(name))
                    .build()?,
            ))
        })
        .collect()
}
