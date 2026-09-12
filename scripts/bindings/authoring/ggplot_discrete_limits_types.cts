import * as c from '../../../target/ggplot-discrete-limits/wasm-module/authoring.cjs';
c.xAxis().continuousLimits([0, {number:'Infinity'}]);
c.y_axis().continuous_limits([true, false]).continuousLimits(null);
c.xAxis().continuousLimits([]).continuousLimits([2]).continuousLimits([-2,1,4]);
