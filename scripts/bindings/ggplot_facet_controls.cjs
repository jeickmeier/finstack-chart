// GG12 independent actual WASM facet authors and publication replay.
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const ROOT=path.resolve(__dirname,'../..'),c=require(path.resolve(process.argv[2],'authoring.cjs'));
const out=path.resolve(process.argv[3]);fs.mkdirSync(out,{recursive:true});
const output=new c.Output(fs.readFileSync(path.join(ROOT,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const levels=values=>values.map(Text=>({Text}));
for(let mode=0;mode<17;mode++){
 const data=c.Data.columns({x:new Float64Array([1,2,1,4,1,10,2]),y:new Float64Array([1,3,2,8,10,100,4]),r:['B','B','A','A','B','B',null],nested:['u','u','v','v','w','w','u'],c:['L','L','R','R','R','R','L'],z:['z1','z2','z1','z2','z1','z2','z1']});
 let b=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()),policy={},facet=c.facet_wrap('r').fields(['r','c']).columns(2),annotation;
 if(mode===1)Object.assign(policy,{drop:false,levels:[levels(['B','A','unused']),levels(['R','L','unused'])]});
 else if(mode===2){facet=c.facet_grid('r','c').fields(['r','nested','c','z']);policy.row_fields=2;}
 else if(mode===3){facet=c.facet_grid('r','c').fields(['r','nested','c']);Object.assign(policy,{row_fields:2,margins:[0,1,2]});}
 else if(mode===4){policy.shrink=false;facet=c.facet_wrap('c').free_y(true);b=b.layer(c.points().stat(c.summary().x(1).y('y').summary_helper({MeanSe:{mult:1}})).color('#DC2828'));}
 else if(mode===5){facet=c.facet_grid('r','c').free_x(true).free_y(true);policy.space='Free';}
 else if(mode===6)policy.direction='Tr';else if(mode===7)policy.direction='Bl';
 else if(mode>=8&&mode<12)policy.strip_position=['Top','Bottom','Left','Right'][mode-8];
 else if(mode===12)Object.assign(policy,{axes:'All',axis_labels:'Margins'});
 else if(mode===13)policy.labeller={variable_names:true,wrap_width:12,lookup:{r:{B:'Business group B'}}};
 else if(mode===14){facet=c.facet_grid('r','c');annotation=c.Data.columns({r:['C'],y:new Float64Array([9]),x:new Float64Array([9])},{name:'annotation'});b=b.layer(c.points().data(annotation).color('#DC2828'));}
 else if(mode===15){facet=c.facet_grid('r','c');Object.assign(policy,{as_table:false,switch:'Both'});}
 else if(mode===16){facet=c.facet_grid('r','c').fields(['c']).free_x(true);Object.assign(policy,{row_fields:0,space:'FreeX'});}
 const p=b.facet(facet.reference(policy)).build(),wire=p.to_json();assert.ok(JSON.parse(wire).version>=76);
 const q=c.Plot.from_json(wire),request=output.request(q,c.export_options(800,560).dpi(144).layout(c.layout_options().minimum_plot([0.1,0.1]))),frame=request.prepare(),scene=frame.scene();
 const directRequest=output.request(p,c.export_options(800,560).dpi(144).layout(c.layout_options().minimum_plot([0.1,0.1]))),direct=directRequest.prepare();assert.deepEqual(direct.scene(),scene);direct.dispose();directRequest.dispose();
 fs.writeFileSync(path.join(out,`facet-${mode}.plot.json`),wire);fs.writeFileSync(path.join(out,`facet-${mode}.scene.json`),JSON.stringify(scene));for(const fmt of ['svg','pdf','png'])fs.writeFileSync(path.join(out,`facet-${mode}.${fmt}`),frame.export(fmt));
 frame.dispose();request.dispose();q.dispose();p.dispose();data.dispose();annotation?.dispose();
}
output.dispose();console.log('PASS WASM GG12 facets: seventeen authors, original/replay equality, 51 publications.');
