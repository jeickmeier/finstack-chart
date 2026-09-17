'use strict';
const fs=require('node:fs'),path=require('node:path');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<8;mode++){
 const position=mode===1?'plot':'panel',[location,tag]=({2:['margin','topleft'],3:['margin','bottomright'],4:['panel','bottomleft'],5:['panel','top'],6:['plot','topright']})[mode]??['plot','topright'];
 const elements=Object.fromEntries(['plot.title.position','plot.caption.position'].map(name=>[name,{Value:{Text:position}}]));
 elements['plot.tag.location']={Value:{Text:location}};elements['plot.tag.position']={Value:mode===5?{Vector:[{Number:.5},{Number:.5}]}:{Text:tag}};if(mode===7)elements['plot.tag']='Blank';
 const data=c.Data.columns({x:c.column([0,1,2,3],{kind:'float64'}),y:c.column([1,3,2,4],{kind:'float64'}),group:['a','a','b','b']});
 let builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).theme(c.theme().reference_preset('Grey',{}).update_elements({elements})).title(c.title('Figure alignment')).subtitle(c.subtitle('Explicit panel or plot span')).caption(c.caption('Caption aligned to the same selected span')).tag(c.rich_text('A'));
 if(mode===6)builder=builder.facet(c.facet_wrap('group'));
 const p=builder.build(),wire=p.to_json(),restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare();
 fs.writeFileSync(path.join(out,`furniture-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`furniture-${mode}.scene.json`),JSON.stringify(frame.scene()));
 for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`furniture-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM furniture controls: eight authors, 24 publications');
