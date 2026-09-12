'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(640,360).dpi(96).basis('current'),records=[];
for(const [index,testcase] of JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/line-widths.json'))).cases.entries()){
 if(testcase.kind==='polygon')continue;
 for(const mapped of [false,true]){
  const owned=[];
  try{
   const kind=testcase.kind,width=testcase.linewidth,data=c.Data.columns({x:[1,2,3],y:[1,2,1],xmax:[1.2,2.2,3.2],ymax:[1.5,2.5,1.5],width:[width,width,width]});owned.push(data);
   const layer=()=>{let layer=(kind==='segment'?c.rule():kind==='rect'?c.rectangle():c.line().order(kind==='path'?'Authored':'X')).name('marks').stroke('#ff0000');return mapped?layer.shape_value('StrokeWidth','width'):layer.linewidth(width);};
   const p=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').x2('xmax').y2('ymax')).layer(layer()).build();owned.push(p);
   const wire=p.to_json(),restored=c.Plot.from_json(wire);owned.push(restored);assert.equal(restored.to_json(),wire);
   for(const state of ['original','edited']){
    const current=state==='original'?restored:restored.edit().layer('marks',layer()).build();if(current!==restored)owned.push(current);
    const request=output.request(current,options);owned.push(request);const frame=request.prepare();owned.push(frame);const svg=frame.export('svg'),text=Buffer.from(svg).toString('utf8');
    const widths=[...text.matchAll(/<[^>]+stroke="#ff0000"[^>]*>/g)].map(m=>Number(m[0].match(/stroke-width="([^"]+)"/)[1]));
    const expected=width===0?testcase.pdf_widths[0]:testcase.lwd[0]*72/96;assert.ok(widths.length>0&&widths.every(v=>Math.abs(v-expected)<2e-12),`${index} ${mapped} ${widths} ${expected}`);
    records.push({index,mapped,state,widths});
    if(width===.5&&mapped&&state==='original'){fs.writeFileSync(path.join(out,`width-${kind}.svg`),svg);for(const fmt of ['pdf','png'])fs.writeFileSync(path.join(out,`width-${kind}.${fmt}`),frame.export(fmt));}
   }
  }finally{for(const value of owned.reverse())value.dispose();}
 }
}
assert.equal(records.length,96);fs.writeFileSync(path.join(out,'line-width-records.json'),JSON.stringify(records,null,2));options.dispose();output.dispose();console.log('PASS WASM: 96 physical linewidth states and twelve publication files.');
