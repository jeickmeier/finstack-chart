import finstack_chart as c

number: c.Interpolator[float] = c.interpolate_number(2, 10)
value: float = number.sample(0.5)
samples: list[float] = c.quantize(number, 3)
rounded: float = c.interpolate_round(-1, 0)(0.5)
text: str = c.interpolate_string("2px", "10px")(0.5)
date: c.InterpolationDate = c.interpolate_date(c.date_value(0), c.date_value(1000))(0.5)
array: c.NumericArray = c.interpolate_number_array([0, 1], c.numeric_array("Uint8ClampedArray", [2, 3]))(0.5)
rgb: str = c.interpolate_rgb.gamma(2.2)("red", "blue")(0.5)
piece: c.Interpolator[float] = c.piecewise(c.interpolate_number, [0, 10, 0])
color_piece: c.Interpolator[str] = c.piecewise(c.interpolate_rgb.gamma(2), ["red", "blue", "white"])
label: str = c.interpolate_discrete(["one", "two"])(0.5)
transform: str = c.interpolate_transform_css("none", "translate(10px, 20px)")(0.5)
matrix: list[float] = c.interpolate_transform_svg([1, 0, 0, 1, 0, 0], [1, 0, 0, 1, 10, 20]).sample_transform(0.5)
zoom: list[float] = c.interpolate_zoom.rho(1)([0, 0, 10], [0, 0, 1])(0.5)
duration: float | None = c.interpolate_zoom([0, 0, 10], [0, 0, 1]).duration

catalog: c.ChromaticCatalog = c.chromatic_catalog()
named: c.Interpolator[c.ColorValue] = c.chromatic('Viridis', reverse=True)
colors: list[c.ColorValue] = c.chromatic_scheme('Blues', 3)
palette: c.SchemeSpec = {'id': 'Blues', 'size': 3, 'reverse': True}
c.color_mapped('q', c.StandaloneScale('quantile', range=['red'])).palette_scheme(palette)
