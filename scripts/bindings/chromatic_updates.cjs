'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const root=path.resolve(__dirname,'../..'),c=require(path.join(path.resolve(process.argv[2]),'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(path.dirname(out),{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))),options=c.export_options(480,260).dpi(72).basis('current'),base=9007199254740992n,records=[];
const data=rows=>c.Data.columns({key:c.column(rows.map(r=>r[1]),{kind:'uint64'}),value:rows.map(r=>r[2]),panel:c.categorical(rows.map(r=>r[3]))},{keys:rows.map(r=>r[0]),name:'live'});
function author(rows,classifier,facets){
 const d=data(rows),scale=new c.StandaloneScale(classifier,classifier==='sequential_quantile'?{interpolator:c.chromatic('Viridis')}:{range:['black']});
 let mapping=c.color_mapped('shared',scale,'Eligible');if(classifier!=='sequential_quantile')mapping=mapping.paletteScheme({id:'Blues',size:3});
 let p=c.plot(d).aes(c.aes().x('value').y('value').color(classifier==='ordinal'?'key':'value').color_scale('shared')).layer(c.points())
 .scale(mapping).x_axis(c.x_axis().scale(c.scale_linear().domain(0,100))).y_axis(c.y_axis().scale(c.scale_linear().domain(0,100)));
 if(facets)p=p.facet(c.facet_wrap('panel').order([['A'],['B']]).columns(2).free_y(true));const result=p.build();d.dispose();scale.dispose();return result;
}
for(const classifier of ['ordinal','quantile','sequential_quantile'])for(const facets of [false,true]){
 const rows=[[1n,base+1n,0,'A'],[2n,base+2n,2,'B'],[3n,base+1n,4,'A']];
 const p=author(rows,classifier,facets),chart=p.chart();chart.present(output,options).dispose();p.dispose();
 const old=chart.request(output,options.basis('presented'));let frame=old.prepare();const png=frame.export('png');frame.dispose();
 for(let step=0;step<4;step++){
  let tx=chart.transaction().id(`scale-${step}`);
  if(step===0){const r=[4n,base+3n,100,'B'];rows.push(r);const batch=data([r]);tx=tx.append('live',batch);batch.dispose();}
  else if(step===1){const r=[2n,base+3n,8,'B'];rows[1]=r;const batch=data([r]);tx=tx.upsert('live',batch);batch.dispose();}
  else if(step===2){rows.shift();tx=tx.remove('live',[1n]);}
  else{rows.shift();tx=tx.retain_count('live',2);}
  tx=tx.build();assert('Applied'in chart.commit(tx));tx.dispose();
  const before=old.prepare();assert.deepEqual(before.export('png'),png);before.dispose();
  const current=chart.request(output,options),freshPlot=author(rows,classifier,facets),fresh=output.request(freshPlot,options);freshPlot.dispose();
  const a=current.prepare(),b=fresh.prepare();assert.deepEqual(a.export('png'),b.export('png'),`${classifier}/${facets}/${step}`);
  for(const v of [a,b,current,fresh])v.dispose();
 }
 chart.dispose();frame=old.prepare();assert.deepEqual(frame.export('png'),png);frame.dispose();old.dispose();
 records.push({family:classifier,facets,updates:4,fresh_batch_rgba_equal:true,retained_capture:true});
}
fs.writeFileSync(out,JSON.stringify(records,null,2)+'\n');console.log('PASS CP-05 WASM: twenty-four append/upsert/remove/retention steps match fresh batch PNG; exact uint64 categories and old captures survive.');
