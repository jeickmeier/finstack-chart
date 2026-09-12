import * as c from '../../../target/ggplot-minor/wasm-module/authoring.cjs';
const numeric: c.MinorBreaks = {Numeric: [0, .5, {number: 'NaN'}]};
const time: c.MinorBreaks = {TimeWidth: '1 month'};
const expansion: c.GgplotExpansion = {mult: [.05, .05], add: [0, 0]};
c.xAxis().minorBreaks(numeric).expansion(expansion);
c.y_axis().minor_breaks(time).expansion(null);
c.axisGuide('minor', 'x').minorBreaks('Automatic').minor_breaks('Hidden').minorBreaks(null);
c.exportOptions(640, 360).layout(c.layoutOptions().maxTicks(4096));
c.xAxis().minorBreaks({Timestamps: [null, {Timestamp:{value:'1700000000000000',unit:'Microseconds'}}]});
