// FIX-GG15 independent WASM geography authors.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const columns=(data,options)=>c.Data.columns(Object.fromEntries(Object.entries(data).map(([k,v])=>[k,v.every(x=>typeof x==='number')?new Float64Array(v):v])),options);
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const ring=(x,y,w,h)=>[[x,y],[x+w,y],[x+w,y+h],[x,y+h],[x,y]];
for(let mode=0;mode<19;mode++){
 let polygons=[[ring(-20,10,30,40),ring(-10,20,10,15)],[ring(20,15,15,20)]];
 if(mode===3)polygons=polygons.map(poly=>poly.map(r=>r.map(([x,y])=>[y,x])));
 const collection={features:[{id:{Text:'land'},geometry:{MultiPolygon:polygons},crs:null}],crs:'Wgs84',axis_order:mode===3?'YX':'XY'};
 if(mode===12)collection.features[0].geometry={Polygon:[[[170,10],[-170,10],[-170,40],[170,40],[170,10]],[[160,20],[-160,20],[-160,30],[160,30],[160,20]]]};
 if(mode===16)collection.features.push({id:{Text:'city'},geometry:{Point:[1113194.9079327357,5621521.486192066]},crs:'WebMercator'});
 const data=columns({id:['land',mode===16?'city':'unmatched'],value:[1,2],x:[-30,45],y:[5,60]});let layer=c.points().geography(collection,'id');
 const operations={4:'Centroid',5:'PointOnSurface',6:'Borders',7:'Coordinates',14:'PointOnSurface',15:'PointOnSurface'};
 if(operations[mode])layer=layer.geography_operation(operations[mode]);
 if([14,15].includes(mode))layer=layer.text_geom(mode===15?{fill:{red:255,green:255,blue:220,alpha:255}}:{}).text_label('id');
 if(mode===11)layer=layer.fill('#40a0d0').color('#603030').linewidth(.8).alpha(.5);
 let projection;
 if([1,4,5,10,11,12,13,14,15].includes(mode))projection={Crs:'WebMercator'};
 else if(mode===2)projection={Crs:{Proj:'+proj=utm +zone=31 +datum=WGS84 +units=m +no_defs'}};
 else if([8,9].includes(mode))projection={Mapproj:{method:mode===8?'mollweide':'tetra',parameters:[],orientation:[90,0,0]}};
 else projection={Crs:'Wgs84'};
 const coordinate={projection,default_crs:'Wgs84'};
 if(mode===10)Object.assign(coordinate,{limits_method:'GeometryBounds',view:{xlim:[{Number:-5},{Number:5}]}});
 if(mode===17)coordinate.graticule={longitude:[-20,0,20],latitude:[10,30,50],label_axes:['Longitude','Latitude','Longitude','Latitude']};
 if(mode===18)coordinate.graticule={datum:null};
 let builder=c.plot(data).profile('Ggplot2_4_0_3').layer(layer).coordinate({Geographic:coordinate});
 if(mode===13)builder=builder.layer(c.line().aes(c.aes().x('x').y('y')).color('red'));
 if(mode>=17)builder=builder.theme(c.theme().reference_preset('Grey',{}));
 const p=builder.build(),wire=p.to_json(),restored=c.Plot.from_json(wire),request=output.request(restored,c.export_options(600,360).dpi(mode===12?300:144)),frame=request.prepare(),scene=frame.scene();
 const originalRequest=output.request(p,c.export_options(600,360).dpi(mode===12?300:144)),originalFrame=originalRequest.prepare();assert.deepEqual(originalFrame.scene(),scene);originalFrame.dispose();originalRequest.dispose();
 fs.writeFileSync(path.join(out,`geography-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`geography-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`geography-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();restored.dispose();p.dispose();data.dispose();
}
output.dispose();console.log('PASS WASM geography controls: 19 authors, 57 publications.');
