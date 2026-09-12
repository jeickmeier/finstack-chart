"""FIX-GG04 checked minor-break and scale-expansion authoring surface."""
import finstack_chart as c
numeric: c.MinorBreaks = {"Numeric": [0., .5, {"number": "NaN"}]}
time: c.MinorBreaks = {"TimeWidth": "1 month"}
expansion: c.GgplotExpansion = {"mult": [.05, .05], "add": [0., 0.]}
c.x_axis().minor_breaks(numeric).expansion(expansion)
c.y_axis().minor_breaks(time).expansion(None)
c.axis_guide("minor", "x").minor_breaks("Automatic").minor_breaks("Hidden").minor_breaks(None)
c.export_options(640, 360).layout(c.layout_options().max_ticks(4096))
c.x_axis().minor_breaks({"Timestamps": [None, {"Timestamp": {"value": "1700000000000000", "unit": "Microseconds"}}]})
