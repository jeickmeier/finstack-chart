import finstack_chart as c
c.chromatic('ViridisTypo')
c.chromatic('Category10')
c.chromatic_scheme('Blues', '3')
c.color_mapped('x', c.StandaloneScale('ordinal', range=['red'])).palette_scheme({'id': 'Blues', 'version': 2, 'size': 3})
