import {ShapeLine,ShapeArea,ShapeLineConfig,ShapeAreaConfig,CurveSpec,Path,shapeLine,shapeArea} from '../../../packages/wasm/authoring.cjs';
const curve:CurveSpec={kind:'CatmullRom',alpha:0.5};
const line:ShapeLineConfig={curve,defined:[true,false],x:{Column:0},y:{Constant:1},digits:null};
const area:ShapeAreaConfig={curve,x0:{Column:0},y0:{Constant:0},x1:null,y1:{Column:1}};
const p:Path=new ShapeLine(line).generate([[1,2],[2,3]]);
new ShapeArea(area).boundary('X1').copy().generate([[1,2]]);
shapeLine().curve(curve);shapeArea().curve({kind:'Natural'});
// @ts-expect-error exact curve parameter surface
new ShapeLine({curve:{kind:'Basis',tension:1}});
// @ts-expect-error unknown family
new ShapeArea({curve:{kind:'Fake'}});
// @ts-expect-error defined is a boolean mask
new ShapeLine({defined:[1,0]});
// @ts-expect-error coordinate columns are numeric indices
new ShapeLine({x:{Column:'x'}});
// @ts-expect-error boundary is a fixed control
new ShapeArea().boundary('Upper');
p.free();
