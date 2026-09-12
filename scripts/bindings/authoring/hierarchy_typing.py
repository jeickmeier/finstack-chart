"""HIR-07 positive static contracts; runtime descriptor controls are separately executed."""
import finstack_chart as c

def hierarchy(data: c.Data, output: c.Output, previous: c.FigureSnapshot) -> c.FigureSnapshot:
    layout: c.HierarchyLayout = {"Treemap": {"options": {"tile": {"Resquarify": 1.6}}, "history": True, "padding_sides":{"Top":"Depth"}}}
    recipe: c.HierarchyRecipe = {"identity":"991", "source":{"Paths":"path"}, "aggregation":{"Sum":"v"}, "label":None, "layout":layout}
    layer = c.hierarchy(recipe).hierarchy_value(data.field("v")).hierarchy_label(data.field("path"))
    layer = layer.hierarchy_projection("Cartesian").hierarchy_order("ValueDescending").hierarchy_limits({"max_nodes":1000})
    plot = c.plot(data).layer(layer).build()
    return output.request(plot, c.export_options(500.,300.)).with_hierarchy_history(previous).prepare()

def operations(envelope: c.HierarchyEnvelope) -> c.HierarchyNodeRecord:
    tree = c.Hierarchy(envelope).sum({"Field":"value"}).layout({"Tree":{"options":{"mode":{"NodeSize":(20.,40.)}}}})
    nodes: list[c.HierarchyNodeRecord] = tree.nodes()
    root = nodes[0]["handle"]
    copy = tree.copy_subtree(root,"12345")
    restored = c.Hierarchy.from_json(copy.to_json())
    return restored.node(restored.nodes()[0]["handle"])
