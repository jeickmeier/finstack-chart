// Execute the same held-capture trace through real Node WebAssembly bindings.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
module.exports=({bindings,root,output})=>{
 const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures/live-export/replay.json'),'utf8'));
 const chart=new bindings.Chart(JSON.stringify(fixture.chart),JSON.stringify(fixture.data),fs.readFileSync(path.join(root,'fixtures/live-export/profile.json'),'utf8'),Uint8Array.from(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))));
 let stamp=JSON.parse(chart.present()).stamp,disposed=false;const jobs={},outputs={},trace=[];
 for(const step of fixture.steps){let result;try{switch(step.kind){
  case 'action':{const state=JSON.parse(chart.state());result=JSON.parse(chart.dispatch(JSON.stringify({definition_revision:state.definition_revision,expected_state:state.state_revision,origin:'Control',scene:stamp,action:step.action})));break;}
  case 'transaction':result=JSON.parse(chart.transaction(JSON.stringify(step.transaction)));break;
  case 'present':stamp=JSON.parse(chart.present()).stamp;result={stamp};break;
  case 'begin':result=JSON.parse(chart.export_control(JSON.stringify({version:1,operation:{Begin:step.options}})));jobs[step.name]=result.job;break;
  case 'cancel':result=JSON.parse(chart.export_control(JSON.stringify({version:1,operation:{Cancel:{job:jobs[step.job]}}})));break;
  case 'export':outputs[step.name]=Uint8Array.from(chart.export_job(jobs[step.job]));result={bytes:outputs[step.name].length};break;
  case 'status':result=JSON.parse(chart.export_control('{"version":1,"operation":"Status"}'));break;
  case 'dispose':chart.dispose();disposed=true;result={disposed:true};break;
  default:throw Error(step.kind);
 }assert(!step.error,step.name);}catch(error){const code=JSON.parse(error.message).code;assert.equal(code,step.error,step.name);result={error:code};}
 const status=disposed?null:JSON.parse(chart.export_control('{"version":1,"operation":"Status"}'));
 const live=disposed?null:{store:JSON.parse(chart.semantics()).store_revision,state:JSON.parse(chart.state())};trace.push({name:step.name,result,status,live});
 }
 for(const[name,data]of Object.entries(outputs)){assert(Buffer.from(data).includes('<svg'));fs.writeFileSync(path.join(output,`live-${name}.svg`),data);}
 fs.writeFileSync(path.join(output,'live-export.json'),JSON.stringify(trace));chart.free();console.log(`PASS WASM FIX-14 ${trace.length} held-capture steps and bytes after disposal`);
};
