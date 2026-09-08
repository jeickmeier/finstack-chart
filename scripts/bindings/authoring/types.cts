import {Data, Plot, plot, aes, line, title, subtitle, labels, legend, xAxis, exportOptions, type DispatchOutcome} from '../../../target/authoring/wasm-module/authoring.cjs';
const data = Data.columns({x:[1,2,3], y:[2,4,3]}, {keys:[1n,2n,3n]});
const p: Plot = plot(data).aes(aes().x(data.field('x')).y('y')).layer(line()).layer(labels().at(2,4).text('Peak')).title(title('Prices')).subtitle(subtitle('Daily')).xAxis(xAxis().label('Time')).build();
const action: DispatchOutcome = p.chart().legendVisible(false);
const width: number = exportOptions(300,200).dpi(96) ? 300 : 0;
void [action,width,p.edit().title(title('Edited')).build()];
// @ts-expect-error Titles must use their figure component slot.
labels().title('Wrong');
// @ts-expect-error An annotation cannot be installed as a title.
plot(data).title(labels());
// @ts-expect-error Subtitles and titles have separate nominal components.
plot(data).title(subtitle('Wrong'));
// @ts-expect-error Legends have their own route.
plot(data).layer(legend());
// @ts-expect-error Definition edits cannot replace historical data.
p.edit().data(data);

import {densityHistogram,withExtensions,chamferedBars} from '../../../target/authoring/wasm-module/examples.cjs';
withExtensions(plot(data)).layer(chamferedBars().stat(densityHistogram(data.field('x'),[0,1,2]))).build();
