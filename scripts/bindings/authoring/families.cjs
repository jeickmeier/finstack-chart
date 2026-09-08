'use strict';
// Ordinary JavaScript authoring for the independent statistic and geometry corpus.
const fs=require('node:fs'),path=require('node:path');
exports.dataset=(c,fixture,index=0)=>{
  const batch=fixture.data.datasets[index].batch,columns={};
  batch.fields.forEach((field,i)=>{
   const wire=batch.columns[i],[kind,values]=Object.entries(wire.values)[0];let col;
   if(kind==='Categorical')col=c.categorical(values.codes.map(i=>values.dictionary[i]));
   else if(kind==='Timestamp')col=c.timestamps(values.map(BigInt),{Seconds:'s',Milliseconds:'ms',Microseconds:'us',Nanoseconds:'ns'}[field.kind.Timestamp.unit],field.kind.Timestamp.timezone);
   else {const kinds={Float64:'float64',Int64:'int64',UInt64:'uint64',Boolean:'bool',Utf8:'string'};col=c.column(['Int64','UInt64'].includes(kind)?values.map(BigInt):values,{kind:kinds[kind]});}
   col=col.validity(wire.validity).nullable(field.nullable);
   if(wire.formatted)col=col.formatted(wire.formatted);
   if(field.unit)col=col.unit(field.unit);
   if(field.label)col=col.label(field.label);
   columns[field.name]=col;
  });
  const source=c.Data.columns(columns,{name:`data_${index}`,keys:batch.keys.map(BigInt),schemaVersion:BigInt(batch.schema_version),identity:BigInt(fixture.data.datasets[index].id)});for(const col of Object.values(columns))col.free();return source;
 };

exports.run=(c,root,output,options,write)=>{
 const dataset=(fixture,index=0)=>exports.dataset(c,fixture,index);
 const gray='#b4c8d78c',blue='#1e7db4d2',red='#dc4637d2',missing='#80808046',semantics={},scenes={};
 const bold=output.registerFont(fs.readFileSync(path.join(root,'fixtures/composition/fonts/NotoSans-Bold.ttf'))),arabic=output.registerFont(fs.readFileSync(path.join(root,'fixtures/composition/fonts/NotoSansArabic-Regular.ttf')));
 for(const family of ['statistics','families','facets','composition'])for(const fixture of JSON.parse(fs.readFileSync(path.join(root,`fixtures/${family}/portable-cases.json`),'utf8'))){
  const name=fixture.name,source=dataset(fixture);let p=c.plot(source),layer;
  if(family==='statistics'){
   if(['summary','transformed-summary'].includes(name)){
    let stat=c.summary().x('x').quantiles([0,.25,.5,1]).emptySumZero(false);if(name==='transformed-summary')stat=stat.transform(2,10);
    layer=c.points().stat(stat).afterStat(c.statAes().x('Group').y('Mean'));
   }else if(name==='count')layer=c.points().stat(c.count().required(['x','y']).group('group')).afterStat(c.statAes().x('Group').y('Count'));
   else if(['ols','filtered-fit'].includes(name)){layer=c.line().stat(c.fit().x('x').y('y')).afterStat(c.statAes().x('X').y('Y'));if(name==='filtered-fit')layer=layer.filter(c.filter('x').maximum(2));}
   else if(name==='auto-bin')layer=c.histogram().aes(c.aes().x('x')).bins(30);
   else if(name==='overflow-bin')layer=c.rectangle().stat(c.bin().x('x').breaks([0,1,2]).outliers('Overflow')).afterBin(c.binAes().x('Start').x2('End').y('Count').y2(0));
   else if(['stack','normalize'].includes(name))layer=c.rectangle().aes(c.aes().x(0).x2(1).y('y').y2(0).group('group')).position(c.stack([0,1,2,3]).normalize(name==='normalize'));
   else if(name==='dodge')layer=c.rectangle().aes(c.aes().x('category').x2('category').y('y').y2(0).group('group')).position(c.dodge([0,1,2]).width(.9));
   else layer=c.points().aes(c.aes().x('x').y('y').group('group')).position(c.jitter((1n<<64n)-1n).displacement(.25,.5).units(name==='jitter-data'?'Data':'Display'));
   p=p.layer(layer.independent().color(gray));
  }else if(family==='families'){
   p=p.aes(c.aes().x('field_1').y('field_2'));
   if(name==='family-log-gaps'){p=p.yAxis(c.yAxis().scale(c.scaleLog(10)));layer=c.line();}
   else if(name==='family-symlog'){p=p.yAxis(c.yAxis().scale(c.scaleSymlog(2)));layer=c.points();}
   else if(name==='family-point-color'){
    p=p.xAxis(c.xAxis().scale(c.scalePoint().categories(['Alpha','Beta','Gamma']).pointPadding(.5))).scale(c.colorDiscrete('color').domain(['Alpha','Beta','Gamma']).palette([blue,red,missing]).missing(missing)).legend(c.legend().scale('color').untitled());
    layer=c.points().aes(c.aes().color('field_1').colorScale('color'));
   }else if(name==='family-area')layer=c.area().baseline(0);
   else if(name==='family-ribbon')layer=c.ribbon().aes(c.aes().y2('field_3'));
   else if(name==='family-heatmap'){
    p=p.scale(c.colorContinuous('color',0,5).palette([blue,red]).clamp(true).missing(missing)).legend(c.legend().scale('color').untitled());
    layer=c.cells().aes(c.aes().x2('field_3').y2('field_4').color('field_5').colorScale('color'));
   }else if(['family-grouped-bars','family-stacked-bars'].includes(name)){
    p=p.aes(c.aes().y('y').y2(0).group('group')).scale(c.colorDiscrete('color').palette([blue,red,missing]).missing(missing)).legend(c.legend().scale('color').untitled());
    layer=name==='family-grouped-bars'?c.rectangle().aes(c.aes().x('category').x2('category')).position(c.dodge([0,1,2]).width(.9)):c.rectangle().aes(c.aes().x(0).x2(1)).position(c.stack([0,1,2,3]));
    layer=layer.colorGroup('color').color(gray);
   }else if(name==='family-ohlc-volume'){
    p=p.yAxis(c.yAxis().name('volume').side('Right')).layer(c.ohlc().width(12).aes(c.aes().y2('field_3').low('field_4').high('field_5')).color(blue));
    layer=c.volume().width(7).axes('x','volume').aes(c.aes().y('field_6')).color(missing);
   }else if(name==='family-secondary'){p=p.yAxis(c.yAxis().name('fahrenheit').side('Right').secondary('y',1.8,32));layer=c.points();}
   else if(['family-session','family-utc-leap'].includes(name)){
    const scale=name==='family-utc-leap'?c.scaleUtc().interval({Days:1}):c.scaleSession({id:'supplied-fixture-only',revision:'1',unit:'Milliseconds',sessions:[{start:'1709164800000',end:'1709251200000'},{start:'1709510400000',end:'1709596800000'}],closed:'Omit'});
    p=p.xAxis(c.xAxis().scale(scale)).aes(c.aes().x({field:'field_1',origin:'1709164800000'}).y('field_2'));layer=c.line();
   }else throw Error(name);
   if(!['family-grouped-bars','family-stacked-bars','family-ohlc-volume'].includes(name))layer=layer.color(blue);
   p=p.layer(layer);
  }
  else if(family==='facets'){
   const panel=text=>({values:[{Text:text}]});
   let facet=c.facetWrap('facet').columns(2).order([['B'],['A']]).gap(12).collectGuides(true).empty('Keep');
   if(name==='facet-free-target')facet=facet.freeY(true);
   if(name==='facet-grid-empty')facet=c.facetGrid('facet','group').order(['B','A','C'].flatMap(a=>[2,1].map(g=>[a,g]))).gap(12).collectGuides(true).empty('Keep');
   p=p.facet(facet);
   if(name.endsWith('summary')){
    const scope={'facet-group-summary':'Group','facet-panel-summary':'Facet','facet-chart-summary':'Chart'}[name];
    layer=c.points().stat(c.summary().x('y').group('group').quantiles([.5]).emptySumZero(false)).afterStat(c.statAes().x('Group').y('Mean')).scope(scope).color(blue);
    if(scope==='Chart')layer=layer.facetTarget('Broadcast');p=p.layer(layer);
   }else{
    p=p.layer(c.points().aes(c.aes().x('x').y('y').group('group').color('group').colorScale('groups')).color(blue)).scale(c.colorDiscrete('groups').domain(['1','2']).palette(['#1e7db4e6','#da6928e6']).missing('#78787864')).legend(c.legend().scale('groups').title('Group'));
    if(name==='facet-shared-log')p=p.yAxis(c.yAxis().scale(c.scaleLog(10)));
    if(fixture.data.datasets.length>1)p=p.layer(c.rule().data(dataset(fixture,1)).aes(c.aes().x(0).x2(2).y('threshold').y2('threshold')).facetTarget(name==='facet-free-target'?{Panels:[panel('A')]}:'Broadcast').color('#8c465adc'));
   }
  }
  else p=composition(c,name,source,dataset(fixture,1),bold,arabic);
  const familyOptions=family==='composition'?c.exportOptions(180,120,'mm').dpi(96):name==='facet-grid-empty'?c.exportOptions(600,540).dpi(96):options;
  const authored=p.build(),chart=authored.chart();semantics[name]=chart.semantics();const frame=chart.present(output,familyOptions);scenes[name]=frame.scene();
  frame.free();chart.free();authored.free();p.free();if(layer)layer.free();source.free();
 }
 write('families',semantics);write('family-scenes',scenes);
};
function composition(c,name,source,reference,bold,arabic){
 const panel=name=>({values:[{Text:name}]}),typography=size=>c.textStyle().size(size),strong=size=>typography(size).font(bold).weight(700),rgb=(red,green,blue)=>({red,green,blue,alpha:255}),terminal=name==='composition-terminal';
 const trace=c.line().aes(c.aes().x('x').y('y')).color('#1e7db4d2'),threshold=c.rule().data(reference).aes(c.aes().x(0).x2(2).y('threshold').y2('threshold')).facetTarget('Broadcast').color('#8c465adc'),observations=c.points().aes(c.aes().x('x').y('y').group('group').color('group').colorScale('groups')).color('#1e7db4d2');
 const theme=c.theme().preset(terminal?'Terminal':name==='composition-grayscale'?'Grayscale':'Editorial').style(c.style().gradient({direction:'Vertical',start:terminal?rgb(27,40,56):rgb(234,243,252),end:terminal?rgb(18,27,37):rgb(255,255,255)})).layer(observations,c.style().symbol('Diamond')).layer(threshold,c.style().dashes([4,2])).layer(trace,c.style().mark(terminal?rgb(117,187,242):rgb(35,93,145)).strokeWidth(1.5));
 return c.plot(source).layer(trace).layer(threshold).layer(observations)
  .scale(c.colorDiscrete('groups').domain(['1','2']).palette(['#1e7db4e6','#da6928e6']).missing('#78787864')).legend(c.legend().scale('groups').title('Group'))
  .facet(c.facetWrap('facet').columns(2).order([['B'],['A']]).gap(12))
  .xAxis(c.xAxis().textStyle(typography(.7).tabular(true)).richLabel(c.richText('Observation x').style(typography(.75))).rotation(-35).format(c.numberFormat().precision(1)))
  .yAxis(c.yAxis().textStyle(typography(.7).tabular(true)).richLabel(c.richText('Value (units)').style(typography(.75)).rotation(-90)).format(c.numberFormat().precision(0)))
  .title(c.title('').rich(c.richText('Small multiples').style(strong(1.4)).run(c.textRun('  ·  one prepared population').style(typography(.9)))))
  .subtitle(c.subtitle('').rich(c.richText('Explicit fonts and vector paint   ').style(typography(.72)).run(c.textRun('مرحبا بالعالم').style(typography(.8).language('ar').direction('RightToLeft').fallback(arabic)))))
  .caption(c.caption('Figure 1 · Shared scales; the inset reuses the same source rows.').style(typography(.72)))
  .sourceNote(c.sourceNote('Source: deterministic fixture. No external market data.').style(typography(.64)))
  .footnote(c.footnote('Notes: x/y are source units; dashed rule = supplied reference.').style(typography(.64)))
  .panelLetter(c.panelLetter('(a)').panel(panel('B')).style(strong(.8))).panelLetter(c.panelLetter('(b)').panel(panel('A')).style(strong(.8)))
  .inset(c.inset().id('detail-a').panel(panel('A')).rectangle(.52,.5,.45,.45).layer(trace).layer(observations).xView(0,1).yView(0,4).guides(false))
  .layer(c.callout().label(c.labels().id('peak').at(2,9).panel(panel('A')).text('Peak 9').style(typography(.68)).offset(-43,15).priority(10).collision('Keep')).to({Data:{panel:panel('A'),scales:{x:'0',y:'1'},x:{Number:2},y:{Number:9}}}))
  .layer(c.labels().id('units').figureAt(.73,.95).text('180 × 120 mm').style(typography(.6)).overflow(true))
  .layer(c.labels().id('lambda').outputAt(430,12).text('λ = −0.25').style(typography(.64)).overflow(true))
  .layer(c.labels().id('direct-b').panelAt(panel('B'),.08,.4).text('Panel B').style(typography(.65)).collision('ShiftThenHide').priority(1)).theme(theme);
}
