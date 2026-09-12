"""FIX-GG04 shared positional bin descriptors in the checked Python surface."""
import finstack_chart as c
spec: c.Options = {"bins": {"breaks": {"Nice": 3}, "limits": None, "right": True, "oob": "Squish"}, "transform": "Reverse"}
c.x_axis().scale(c.scale_binned(spec))
