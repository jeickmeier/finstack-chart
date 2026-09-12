'use strict';
const fs=require('node:fs'),path=require('node:path');
const api=require(path.resolve(process.argv[2],'authoring.cjs'));
const registry=api.ShapeRegistry.example();
function run(steps){const sessions=new Map(),out={};try{
  for(const step of steps){const name=step.session,data=step.data;let result;
    switch(step.action){
      case'construct':sessions.set(name,new api.Hierarchy(data,registry));break;
      case'apply':sessions.get(name).apply(data);break;
      case'query':result=sessions.get(name).query(data);break;
      case'tile':result=sessions.get(name).tile(data);break;
      case'packing':result=(data.siblings?api.pack_siblings:api.pack_enclose)(data.circles,data.limits);break;
      case'snapshot':result=JSON.parse(sessions.get(name).to_json());break;
      case'restore':{const old=sessions.get(name);sessions.set(name,api.Hierarchy.from_json(old.to_json(),registry));old.free();break;}
      case'clone':sessions.set(step.target,sessions.get(name).copy());break;
      case'copy_subtree':sessions.set(step.target,sessions.get(name).copy_subtree(data.node,data.identity));break;
      default:throw new Error(step.action);
    }
    if('out'in step)out[step.out]=result;
  }
  return out;
}catch(error){if(error instanceof api.ChartError)return {error:error.message,code:error.code};throw error;}
finally{for(const session of sessions.values())session.free();}}
try{const cases=JSON.parse(fs.readFileSync(process.argv[3],'utf8'));const results=cases.map(c=>({id:c.id,result:run(c.steps)}));fs.writeFileSync(process.argv[4],JSON.stringify(results));console.log(`${results.length} Node WASM hierarchy sequences executed`);}finally{registry.free();}
