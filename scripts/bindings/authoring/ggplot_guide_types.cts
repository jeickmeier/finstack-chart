import * as c from '../../../packages/wasm/authoring.cjs';
const layer: c.Layer = c.points().legend({show: true, key_glyph: 'Point'});
const key: c.Legend = c.legend().aesthetic('Shape').options({reverse: true, ncol: 2});
const custom: c.Legend = c.legend().custom({id: '999', bounds: [0, 0, 40, 20], paths: []});
const axis = c.xAxis().ggplotAxis({n_dodge: 2, check_overlap: true});
const guide = c.axisGuide('outer', 'x').ggplotAxis({logticks: {expanded: false}});
void [layer, key, custom, axis, guide];
