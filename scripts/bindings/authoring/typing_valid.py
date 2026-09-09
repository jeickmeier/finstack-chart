from finstack_chart import Data, Plot, plot, aes, line, title, subtitle, labels, legend, x_axis, export_options, DispatchOutcome

data = Data.columns({'x':[1,2,3], 'y':[2,4,3]}, keys=[1,2,3])
p: Plot = (plot(data).aes(aes().x(data.field('x')).y('y')).layer(line())
           .layer(labels().at(2,4).text('Peak')).title(title('Prices'))
           .subtitle(subtitle('Daily')).x_axis(x_axis().label('Time')).build())
action: DispatchOutcome = p.chart().legend_visible(False)
options = export_options(300,200).dpi(96)
edit: Plot = p.edit().title(title('Edited')).build()
legend().scale('series').untitled().generic_title().title('Series')
# Expected errors are checked separately; this positive file must have no suppression.

from finstack_chart.examples import density_histogram, with_extensions, chamfered_bars
with_extensions(plot(data)).layer(chamfered_bars().stat(density_histogram(data.field("x"),[0.,1.,2.]))).build()

from finstack_chart import Path, path_round, vector_path
path_value: Path = path_round().move_to(0,0).arc_to(10,0,10,10,2).close_path()
path_value.copy().to_svg()
p.edit().annotation(vector_path("curve",path_value).fill(None)).build()
