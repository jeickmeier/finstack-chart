const fs=require('fs'),path=require('path'),assert=require('assert'),root=path.resolve(__dirname,'../..');
const c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),records=[];
for(const test of JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/device-controls.json'))).dimensions){
 try{const plan=output.resolve_save('figure.PNG',test.input);assert.ok(!test.result.error);assert.ok(Math.abs(plan.page.width/72-test.result[0])<1e-12);assert.ok(Math.abs(plan.page.height/72-test.result[1])<1e-12);records.push(plan);}catch(e){assert.ok(test.result.error);records.push({error:true});}
}
const data=c.Data.columns({x:c.column([0,1],{kind:'float64'}),y:c.column([1,2],{kind:'float64'})}),plot=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).build();
const saved=output.save_figure(plot,'nested/fig-%02d.png',{width:144,height:72,units:'px',dpi:72},{pageNumber:3});assert.equal(saved.plan.path,'nested/fig-03.png');assert.deepEqual(Array.from(saved.bytes.slice(0,8)),[137,80,78,71,13,10,26,10]);
const custom=frame=>{assert.ok(frame.scene().items.length);return new TextEncoder().encode('custom-device');};
const options={width:2,height:1,device:'custom'};assert.equal(new TextDecoder().decode(output.save_figure(plot,'custom.dat',options,{device:custom}).bytes),'custom-device');
const sentinel=new Error('custom failure');assert.throws(()=>output.save_figure(plot,'failed.dat',options,{device:()=>{throw sentinel;}}),e=>e===sentinel);assert.throws(()=>output.save_figure(plot,'oversize.dat',options,{device:()=>new Uint8Array(4194305),maxBytes:4194304}),RangeError);
fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));plot.dispose();data.dispose();output.dispose();console.log('PASS WASM save dimensions, filename inference, owned bytes, callback errors and budgets');
