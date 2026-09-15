import * as c from '../../../target/ggplot-position-functions/wasm-module/authoring.cjs';
c.x_axis().limits_function({operation: {id:'example.numeric_limits',version:'1'},parameters:{mode:'identity'}});
c.yAxis().scale(c.scaleSqrt()).limitsFunction({operation:{id:'example.numeric_limits',version:1n},parameters:{mode:'single'}});
c.x_axis().scale(c.scale_binned({transform:'Sqrt'})).limitsFunction({operation:{id:'example.numeric_limits',version:'1'},parameters:{mode:'fixed'}});
c.x_axis().missing_value(5);
c.yAxis().scale(c.scaleSqrt()).missingValue(-1);
c.x_axis().missing_value(null);
c.x_axis().missingValue({number:'NaN'});

c.x_axis().numeric_limits([null, 5]);
c.yAxis().numericLimits([{number:'-Infinity'}, {number:'Infinity'}]);
c.x_axis().numeric_limits(null);
