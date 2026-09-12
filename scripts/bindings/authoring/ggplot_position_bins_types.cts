import * as c from '../../../target/ggplot-positional/wasm-module/authoring.cjs';
const spec: c.Options = {bins: {breaks: {Nice: 3}, limits: null, right: true, oob: 'Squish'}, transform: 'Reverse'};
c.xAxis().scale(c.scaleBinned(spec));
c.y_axis().scale(c.scale_binned(spec));
