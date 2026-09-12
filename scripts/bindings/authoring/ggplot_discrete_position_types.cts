import * as c from '../../../target/ggplot-position-null/wasm-module/authoring.cjs';
c.xAxis().discretePolicy({limits:['Null', {Text:'NA'}], na_translate:true});
c.y_axis().discrete_policy({levels:[{Text:'a'},'Null'],drop:false}).discretePolicy(null);
c.xAxis().discretePolicy({palette:[2,4,{number:'NaN'}]});
