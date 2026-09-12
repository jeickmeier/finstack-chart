"""FIX-GG04 explicit R date labels retain a checked host descriptor."""
import finstack_chart as c
format: c.GuideFormatter = {"GgplotTime": {"pattern": "%F\n%OS6", "locale": None}}
c.x_axis().tick_format(format)
c.x_axis().scale(c.scale_duration()).tick_format({"GgplotTime": {"pattern": "%H:%M"}})
