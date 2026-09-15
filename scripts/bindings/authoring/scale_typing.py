from typing import assert_type
from finstack_chart import StandaloneScale, ScaleKey, CalendarInterval, interpolate_lab, interpolate_rgb, MISSING, InverseExtent, InterpolationResult

numeric = StandaloneScale("log", domain=[1.0, 10.0, 100.0], range=[0.0, 0.5, 1.0], base=2, clamp=True)
assert_type(numeric.invert(0.5), InterpolationResult)
assert_type(numeric.format(10.0, specifier=".2f"), str)
numeric.configure(domain=[1.0, 16.0], round=True).nice(4).dispose()
numeric.map(MISSING)
StandaloneScale("linear", range=["red", "blue"], factory=interpolate_lab)
StandaloneScale("diverging", domain=[-1.0, 0.0, 2.0], interpolator=interpolate_rgb("red", "blue"))
category = StandaloneScale("ordinal", domain=["a", 1.0, 1, ScaleKey("Unsigned", 18446744073709551615)], range=["low", "high"])
category.train(["later"]).map("later")
assert_type(StandaloneScale("quantize").invert_extent(0.0), InverseExtent)
time = StandaloneScale("utc", domain=[1700000000000000001, 1700000000000000101], unit="Nanoseconds")
interval: CalendarInterval = {"unit": "SourceTick", "step": 10}
assert_type(time.offset(1700000000000000001, interval), int)
time.ticks(interval=interval)
time.format(1700000000000000001, pattern="%H:%M:%S.%f")

from finstack_chart import scale_transform, x_axis
x_axis().scale(scale_transform({"Ggplot": {"transform": {"BoxCox": {"p": 0.5, "offset": 2.0}}}}))
