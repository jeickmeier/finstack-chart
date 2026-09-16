'use strict';
// GG06 independently authored primary WASM bins, source controls and replay.
const fs=require('fs'),path=require('path'),root=path.resolve(__dirname,'../..');
const c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const cases=JSON.parse(fs.readFileSync(path.join(root,'fixtures/parity/ggplot2/bin-stat-controls.json'))).cases,records=[];
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(const [i,t] of cases.entries()) {const owned=[];try {
 const data=c.Data.columns({x:t.input,w:t.weight});owned.push(data);const mode=t.mode,opts={closed:t.closed==='right'?'Right':'Left',pad:t.pad};
 if(!['explicit','bins'].includes(mode))opts.binwidth=.75;if(['center','boundary'].includes(mode))opts[mode]=.25;
 let stat=c.bin().bins(3).ggplot_bin(opts).bin_weight('w');if(mode==='explicit')stat=stat.breaks([0,1,2,4]);
 const p=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x')).layer(c.histogram().stat(stat)).build();owned.push(p);
 const wire=p.to_json();if(JSON.parse(wire).version!==72)throw Error('wire');const q=c.Plot.from_json(wire);owned.push(q);if(q.to_json()!==wire)throw Error('replay');
 for(const [state,current] of [['original',p],['replay',q]]) {const chart=current.chart();owned.push(chart);const rows=chart.semantics().layers[0].rows.Binned;if(rows.length!==t.result.length)throw Error('row count');
 const values=rows.map((row,j)=>{const actual={xmin:row.start,xmax:row.end,...row.statistics},e=t.result[j];for(const [key,v] of Object.entries(actual)) {if(v===null?e[key]!==null:Math.abs(v-e[key])>3e-12*Math.max(1,Math.abs(e[key])))throw Error(JSON.stringify({i,key,v,e:e[key]}));}return actual;});records.push({case:i,state,rows:values});}
 const selected=t.closed==='right'&&((t.population==='ordinary'&&!t.pad)||(mode==='explicit'&&['signed','zero'].includes(t.population)&&t.pad))||t.closed==='left'&&t.population==='boundary'&&mode==='explicit'&&t.pad;
 if(selected){const options=c.export_options(480,320);owned.push(options);const request=output.request(q,options);owned.push(request);const frame=request.prepare();owned.push(frame);for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`bin-${i}.${fmt}`),frame.export(fmt));}
 }finally{for(const item of owned.reverse())item.dispose();}}
output.dispose();if(records.length!==200||fs.readdirSync(out).filter(x=>x.endsWith('.svg')).length!==8)throw Error('coverage');fs.writeFileSync(path.join(out,'records.json'),JSON.stringify(records));console.log('PASS GG06 WASM bins: 200 states, 24 publications.');
