'use strict';
const { alphaByte } = require('./ggplot_reference_alpha.cjs');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
function hide(value){if(value&&typeof value==='object'){if(value.guide&&typeof value.guide==='object'&&Object.hasOwn(value.guide,'Temporal'))value.guide='Hidden';for(const child of Object.values(value))hide(child);}}
const records=[];
for(const [index,testcase] of JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/temporal-aesthetic-precision.json'))).cases.entries()){
 const owned=[];
 try{
  const channel=testcase.channel,stamps=testcase.offsets.map(v=>BigInt(testcase.epoch)*1000000000n+BigInt(Math.round(v*1e9)));
  const data=c.Data.columns({x:c.column([0,1,2,3],{kind:'float64'}),y:c.column([0,1,2,3],{kind:'float64'}),when:c.timestamps(stamps,'ns','UTC')});owned.push(data);
  const mapping=c.aes().x('x').y('y')[channel==='colour'?'color':channel]('when');
  const plot=c.plot(data).profile('Ggplot2_4_0_3').aes(mapping).layer(c.points().stroke('#000000')).build();owned.push(plot);
  const wire=JSON.parse(plot.to_json());assert.equal(wire.version,['colour','fill'].includes(channel)?42:45);if(testcase.hidden)hide(wire);
  const restored=c.Plot.from_json(JSON.stringify(wire));owned.push(restored);const chart=restored.chart();owned.push(chart);const styles=chart.semantics().layers[0].styles;assert.equal(styles.length,4);
  styles.forEach((style,i)=>{const value=testcase.values[i];if(channel==='alpha')assert.equal(style.color.alpha,alphaByte(value));else if(['size','linewidth'].includes(channel))assert.ok(Math.abs(style[channel==='size'?'radius':'stroke_width']-value)<2e-12,`${index} ${JSON.stringify(style)} ${value}`);else{const paint=style[channel==='fill'?'fill':'color'];assert.deepEqual([paint.red,paint.green,paint.blue],[1,3,5].map(i=>parseInt(value.slice(i,i+2),16)));}});
  records.push({index,styles});
 }finally{for(const value of owned.reverse())value.dispose();}
}
assert.equal(records.length,180);fs.writeFileSync(path.join(out,'precision-records.json'),JSON.stringify(records,null,2));console.log('PASS WASM: 180 temporal precision cases.');
