'use strict';
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const registered=process.argv.includes('--registered'),registry=registered?c.ExtensionRegistry.example():undefined;
const records=[],compositions=process.argv.includes('--compositions'),version=registered?10:compositions?9:8,transform=registered?{Registered:{selection:{call:{operation:{id:'example.scale_transform',version:'1'},parameters:{family:'cubic',custom:false}}}}}:compositions?{Compose:{transforms:['Asinh','Reverse']}}:'Asinh';
for(const kind of ['Numeric','Continuous','Sequential','Diverging','Ggplot']){
 const base=new c.StandaloneScale(['Numeric','Continuous'].includes(kind)?'linear':kind==='Diverging'?'diverging':'sequential',kind==='Continuous'?{range:['red','blue']}:{}),spec=base.spec();
 if(['Numeric','Continuous'].includes(kind))spec[kind].family={Ggplot:{transform}};
 else if(kind==='Ggplot')spec.Interpolated.normalization={Ggplot:{family:{Ggplot:{transform}},domain:[0,1],reverse:false,rescaler:'Range'}};
 else spec.Interpolated.normalization[kind].family={Ggplot:{transform}};
 const scale=c.StandaloneScale.from_spec(spec,registry),wire=scale.to_json(),envelope=JSON.parse(wire);assert.equal(envelope.version,version);
 const restored=c.StandaloneScale.from_json(wire,registry),copied=restored.copy();assert.equal(restored.to_json(),wire);assert.equal(copied.to_json(),wire);
 for(const value of [0,.25,.5,1]){assert.deepEqual(scale.map(value),restored.map(value));assert.deepEqual(scale.map(value),copied.map(value));if(kind==='Numeric')assert.ok(Math.abs(scale.map(value)-(registered?value**3:Math.asinh(value)/Math.asinh(1)))<2e-15);}
 for(let downgrade=1;downgrade<version;downgrade++){envelope.version=downgrade;assert.throws(()=>c.StandaloneScale.from_json(JSON.stringify(envelope),registry),/version/i);assert.equal(scale.to_json(),wire);}
 if(registered){assert.throws(()=>c.StandaloneScale.from_json(wire),/register/i);const native=c.StandaloneScale.from_spec(JSON.parse(JSON.stringify(spec).replaceAll('example.scale_transform','example.native_scale_transform')),registry);assert.deepEqual(native.map(.5),scale.map(.5));assert.throws(()=>native.to_json(),/native-only/i);native.dispose();assert.throws(()=>c.StandaloneScale.from_spec(JSON.parse(JSON.stringify(spec).replaceAll('example.scale_transform','example.unknown_transform')),registry),/not registered/i);const updated=scale.reconfigure(scale.spec());assert.equal(updated.to_json(),wire);updated.dispose();const configured=scale.configure({domain:scale.domain()});assert.deepEqual(configured.map(.5),scale.map(.5));configured.dispose();}
 records.push({kind,envelope:JSON.parse(wire),downgrades_rejected:version-1});for(const owned of [copied,restored,scale,base])owned.dispose();
}
const legacy=new c.StandaloneScale('linear');assert.equal(JSON.parse(legacy.to_json()).version,1);legacy.dispose();if(registry)registry.dispose();
fs.writeFileSync(path.join(out,'states.json'),JSON.stringify(records,null,2)+'\n');
console.log(`PASS WASM transform wire: five containers, ${5*(version-1)} rejected downgrades, copies/mapping and legacy v1 control.`);
