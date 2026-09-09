import {ShapeRegistry,ShapeOperation,ShapeLine,ShapeArea,ShapeLineRadial,ShapeAreaRadial,ShapeLink,ShapeSymbol,ShapePie,ShapeStack,Data,plot,aes,shapeLine,Plot} from '../../../packages/wasm/authoring.cjs';
const op:ShapeOperation={operation:{id:'example.shift_curve',version:1n},parameters:{amount:2}},registry=ShapeRegistry.example();
const symbol:ShapeOperation={operation:{id:'example.rectangle_symbol',version:1},parameters:{amount:4}},compare:ShapeOperation={operation:{id:'example.field_comparator',version:1},parameters:{field:'rank'}},order:ShapeOperation={operation:{id:'example.first_value_order',version:1},parameters:{}},offset:ShapeOperation={operation:{id:'example.shift_offset',version:1},parameters:{amount:2}};
registry.selection(op,'Curve');new ShapeLine().generateRegistered([[0,1]],registry,op);new ShapeArea().generateRegistered([[0,1]],registry,op);
new ShapeLineRadial().generateRegistered([[0,1]],registry,op);new ShapeAreaRadial().generateRegistered([[0,1]],registry,op);new ShapeLink().generateRegistered({source:[0,1],target:[1,2]},registry,op);
new ShapeSymbol().generateRegistered(registry,symbol);const result=new ShapePie().layoutRegistered([{rank:9007199254740993n}],new Float64Array([1]),registry,compare);const rank:string=result[0].data.rank;
new ShapeStack({keys:['a']}).layoutRegistered([{id:'x'}],[[1]],registry,order,offset);
const p=plot(Data.columns({x:new Float64Array([0]),y:new Float64Array([1])})).withShapeRegistry(registry).aes(aes().x('x').y('y')).layer(shapeLine().shapeProtocol('Curve',op)).build();Plot.fromJson(p.toJson(),registry);
// @ts-expect-error Executable callback is not a JSON parameter.
const invalid:ShapeOperation={operation:{id:'example.bad',version:1},parameters:()=>2};
// @ts-expect-error Protocol family is closed.
registry.selection(op,'UnknownProtocol');
// @ts-expect-error A generator is not a registry.
new ShapeLine().generateRegistered([[0,1]],new ShapeLine(),op);
