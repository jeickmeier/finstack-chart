// GG17 real raster devices through the WASM authoring API.
const fs=require('node:fs'),path=require('node:path');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs')),out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const data=c.Data.columns({x:new Float64Array([0,1,2]),y:new Float64Array([1,3,2])}),p=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.line()).layer(c.points()).build(),output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
for(const dpi of [72,144]){
 const options=c.export_options(360,240).dpi(dpi).raster_device({tiff_compression:'Lzw',tiff_predictor:true}),request=output.request(p,options),frame=request.prepare();
 for(const fmt of ['jpeg','tiff','bmp','png','ps','eps','tex','emf'])fs.writeFileSync(path.join(out,`device-${dpi}.${fmt}`),frame.export(fmt));
 for(const compression of ['Jpeg','Lzma','Zstd','Webp']){
  const extraOptions=c.export_options(360,240).dpi(dpi).raster_device({tiff_compression:compression}),extraRequest=output.request(p,extraOptions),extra=extraRequest.prepare();
  fs.writeFileSync(path.join(out,`codec-${dpi}-${compression.toLowerCase()}.tiff`),extra.export('tiff'));
  extra.dispose();extraRequest.dispose();extraOptions.dispose();
 }
 const pages=frame.pages();pages.append(frame);
 frame.dispose();request.dispose();options.dispose();
 for(const fmt of ['ps','pdf','tiff'])fs.writeFileSync(path.join(out,`pages-${dpi}.${fmt}`),pages.export(fmt));
 pages.dispose();
}
output.dispose();p.dispose();data.dispose();console.log('PASS 30 WASM device publications at 72/144 DPI.');
