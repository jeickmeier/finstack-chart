import {ShapeStack,ShapeStackConfig,StackSeries,shapeStack,shapeArea,bars} from '../../../packages/wasm/authoring.cjs';
const config:ShapeStackConfig={keys:['a','b'],order:{Explicit:[1,0]},offset:'Wiggle',missing:'Gap'},s=new ShapeStack(config),copy=s.copy(),values:StackSeries[]=s.layout([[1,2],[3,4]]);
const materialized=copy.layout([{id:9007199254741001n,nested:[{key:3n}]}],[[1,2]]);
const exactId:string=materialized[0].points[0].data.id, nestedKey:string=materialized[0].points[0].data.nested[0].key;
// @ts-expect-error portable BigInt data becomes exact decimal text
const wrongId:bigint=materialized[0].points[0].data.id;
shapeArea().position(shapeStack(['a','b']).stackOrder('InsideOut').stackOffset('Expand').stackMissing('Zero'));
bars().position(shapeStack([0,1]).stackOrder({Explicit:[1,0]}));
// @ts-expect-error explicit order policy
new ShapeStack({order:'Random'});
// @ts-expect-error explicit offset policy
new ShapeStack({offset:'Normalize'});
// @ts-expect-error explicit missing-cell policy
new ShapeStack({missing:'Ignore'});
// @ts-expect-error permutation is tagged
shapeStack(['a']).stackOrder([0]);
// @ts-expect-error named offset
shapeStack(['a']).stackOffset('Normalized');
// @ts-expect-error named missing policy
shapeStack(['a']).stackMissing(true);
s.free();copy.free();
