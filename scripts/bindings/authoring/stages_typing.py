"""FIX-GG02 positive stage-typed primary consumers."""
import finstack_chart as c

data=c.Data.columns({'x':[1.,2.],'y':[10.,100.]})
fraction=c.bin_expr('Count') / c.bin_expr('Count').sum()
source=c.source_expr(data.field('y'))*2.+1.
plot=(c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y(source))
      .layer(c.points().stat(c.summary().x(source)).after_stat(c.stat_aes().x(1.).y(c.stat_expr('Mean')*2.)))
      .layer(c.histogram().after_bin(c.bin_aes().y(fraction)))
      .layer(c.points().after_scale(c.scale_aes().size(c.after_scale_expr('Size')*2.).color(c.from_theme('Accent'))))
      .theme(c.theme().geometry(accent='#1256ab',point_size=1.5))
      .y_axis(c.y_axis().coordinate_scale(c.scale_log(10)).oob('Keep')).build())
plot.edit().profile('LibraryV1').build()
