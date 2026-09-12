import * as c from '../../../target/ggplot-formats/wasm-module/authoring.cjs';
const format: c.GuideFormatter = {GgplotTime: {pattern: '%F\n%OS6', locale: null}};
c.xAxis().tickFormat(format);
c.xAxis().scale(c.scaleDuration()).tickFormat({GgplotTime: {pattern: '%H:%M'}});
