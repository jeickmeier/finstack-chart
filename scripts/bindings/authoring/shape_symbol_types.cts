import {ShapeSymbol,ShapeSymbolConfig,SymbolKind,Path,Data,shapeSymbol,sourceExpr,StandaloneScale} from '../../../packages/wasm/authoring.cjs';
const cfg:ShapeSymbolConfig={kind:'Star',size:64,digits:3},s=new ShapeSymbol(cfg),p:Path=s.copy().generate(),fill:readonly SymbolKind[]=ShapeSymbol.palettes()[0];
const d=Data.columns({type:['a','b'],area:[1,2]});shapeSymbol().symbolTypes(d.field('type'),['a','b'],['Circle','Plus']).shapeValue('AreaSize',sourceExpr(d.field('area')).mul(64)).symbolTitle('Kind').symbolSizeGuide('Area',[1,2]).symbolPaint('Auto');
shapeSymbol().numericScale('AreaSize',d.field('area'),new StandaloneScale('linear',{domain:[0,2],range:[16,256]})).symbolSizeGuide('Area',[0,1,2]);
// @ts-expect-error unknown type
new ShapeSymbol({kind:'Unknown'});
// @ts-expect-error numeric area
new ShapeSymbol({size:'large'});
// @ts-expect-error explicit paint policy
shapeSymbol().symbolPaint('Closed');
// @ts-expect-error exact category labels
shapeSymbol().symbolTypes('type',[1],['Circle']);
// @ts-expect-error unknown channels are rejected
shapeSymbol().shapeValue('UnknownChannel','area');
// @ts-expect-error palettes are immutable
fill.push('Square');
p.free();s.free();
