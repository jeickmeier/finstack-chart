// FIX-GG11 independent WASM spatial statistic and geometry authors.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const columns=(data,options)=>c.Data.columns(Object.fromEntries(Object.entries(data).map(([k,v])=>[k,v.every(x=>typeof x==='number')?new Float64Array(v):v])),options);
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(let mode=0;mode<16;mode++){
 let data,layer;
 if(mode>=6&&mode<=9){
  const n=mode===9?2:5,x=[],y=[],z=[];
  for(let row=0;row<n;row++)for(let col=0;col<n;col++){const px=col-(n===5?2:0),py=row-(n===5?2:0);x.push(px);y.push(py);z.push(mode===8&&row===2&&col===2?null:mode===9?(row===col?0:2):px*px+py*py);}
  data=columns({x,y,z});const filled=mode===7||mode===8;
  const levels={breaks:mode===9?[.5,1,1.5]:filled?[.5,2,4]:[1,2,4],bins:null,binwidth:null};
  const stat=c.spatial_stat({Contour:{levels,filled}}).input('z');layer=(filled?c.contour_filled():c.contour()).stat(stat);
 }else{
  data=columns({x:[0,.25,.5,1,1.5,2,2],y:[0,.75,1,.5,1.5,2,0],z:[1,2,4,8,16,32,64],w:[1,2,0,3,1,2,1]});
  const axis={breaks:null,bins:30,options:{binwidth:1,boundary:0}};
  if(mode===0||mode===1){let stat=c.spatial_stat({Rectangular:{axes:[axis,axis],summary:mode===1?'Mean':null,drop:false}});layer=c.bin2d().stat(mode===1?stat.input('z'):stat.weight('w'));}
  else if(mode===2||mode===3)layer=c.hex().stat(c.spatial_stat({Hexagonal:{binwidth:[1,1],bins:[30,30],summary:mode===3?'Median':null,drop:true}}).input('z').weight('w'));
  else if([4,5,14,15].includes(mode)){const filled=mode===5||mode===15;const stat=c.spatial_stat({Density:{bandwidth:[1,2],adjust:[1,1],n:[25,25],contour:mode===14?null:{breaks:null,bins:null,binwidth:null},contour_var:mode===15?'Normalized':'Density',filled}}).weight('w');layer=(mode===14?c.bin2d():filled?c.contour_filled():c.density2d()).stat(stat);}
  else if(mode>=10&&mode<=12)layer=c.ellipse().stat(c.spatial_stat({Ellipse:{kind:['Normal','T','Euclidean'][mode-10],level:.8,segments:51}}).weight('w'));
  else layer=c.hex().stat(c.identity_stat()).recipe({Hexagon:{width:.4,height:.3}}).fill('#D4A843');
 }
 const builder=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(layer);
 const p=builder.build(),wire=p.to_json(),restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(144)),frame=request.prepare(),scene=frame.scene();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),scene);originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`spatial-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`spatial-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`spatial-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM spatial controls: sixteen authors, 48 publications.');
