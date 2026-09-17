// FIX-GG10 independent WASM weighted model authors.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const columns=(data,options)=>c.Data.columns(Object.fromEntries(Object.entries(data).map(([k,v])=>[k,v.every(x=>typeof x==='number')?new Float64Array(v):v])),options);
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<10;mode++){
 const data=columns({x:Array.from({length:24},(_,i)=>i),y:Array.from({length:24},(_,i)=>1+i%5+0.2*i),w:Array.from({length:24},(_,i)=>1+i%3)});let method;
 if([0,7,8,9].includes(mode))method='Linear';
 else if(mode===1)method={Glm:{family:'Poisson',epsilon:1e-8,iterations:25}};
 else if(mode===2)method={Loess:{span:0.75,degree:2,cell:0.2,surface:'Interpolate',family:'Gaussian',iterations:4,normalize:true,exact_statistics:false,approximate_trace:false}};
 else if(mode===3)method={Gam:{basis_dimension:10,knots:null,iterations:120,tolerance:1e-9}};
 else if(mode===4||mode===5)method={Quantile:{solver:mode===4?'Br':'Fn',probabilities:[0.25,0.5,0.75],iterations:10000}};
 else method='Auto';
 const options={method,terms:mode===0?[{Power:0},{Power:1},{Power:2}]:null,n:41,xseq:null,full_range:mode===9,se:mode!==4&&mode!==5,level:mode===7?0.5:0.95};
 let layer=c.smooth().stat(c.model_stat(options).x('x').y('y').weight('w'));
 if(mode===8)layer=layer.orientation('Horizontal');
 const builder=c.plot(data).profile('Ggplot2_4_0_3').layer(layer).x_axis(c.x_axis().scale(c.scale_linear().domain(mode===8?-2:-1,mode===8?12:24))).y_axis(c.y_axis().scale(c.scale_linear().domain(mode===8?-1:-2,mode===8?24:12)));
 const p=builder.build(),wire=p.to_json(),restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),scene);originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`model-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`model-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`model-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM model controls: ten authors, 30 publications.');
