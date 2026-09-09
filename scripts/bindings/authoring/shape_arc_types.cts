import {ShapeArc,ShapePie,ArcDatum,ShapeArcConfig,ShapePieConfig,Path,Data,shapeArc,shapePie,sourceExpr} from '../../../packages/wasm/authoring.cjs';
const config:ShapeArcConfig={inner_radius:0,outer_radius:30,start_angle:0,end_angle:1,corner_radius:2,pad_radius:null},arc=new ShapeArc(config),p:Path=arc.generate();
const datum:ArcDatum={inner_radius:5,outer_radius:20,start_angle:0,end_angle:1};arc.centroid(datum);
const pie:ShapePieConfig={order:'Input',angles:{pad_angle:.1}};new ShapePie(pie).layout([1,2]);
const result=new ShapePie().layout([{id:'9007199254741001'}],new Float64Array([1]));const id:string=result[0].data.id;
const data=Data.columns({weight:[1,2]});shapePie().shapeValue('PieValue',data.field('weight')).pieOrder('ValuesDescending').pieAngles({end_angle:6});shapeArc().shapeValue('OuterRadius',sourceExpr(data.field('weight')).mul(2));
// @ts-expect-error radius is a number
new ShapeArc({outer_radius:'x'});
// @ts-expect-error data comparators require the later custom protocol
new ShapePie({order:'DataDescending'});
// @ts-expect-error nonnumeric data requires values
new ShapePie().layout(['a']);
// @ts-expect-error exact named channel
shapePie().shapeValue('Radius','weight');
// @ts-expect-error missing is represented by omission
new ShapeArc().centroid({end_angle:null});
const exactPieId:string=new ShapePie().layout([{id:9007199254741001n}],[1])[0].data.id;
// @ts-expect-error callbacks are not portable materialized data
new ShapePie().layout([{id:()=>1}],[1]);
p.free();arc.free();
