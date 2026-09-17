import * as c from '../../../target/ggplot-packages/wasm-module/authoring.cjs';
c.bin().ggplotBin({closed:'Right',pad:true}).binWeight('w');
c.count().ggplotCount().countWeight('w').countWidth(.9);
c.summary().summaryHelper({MeanSe:{mult:1.}}).summaryBins({bins:3});
c.points().position(c.ggplotStack().vjust(.5).reverse(true));
c.points().position(c.ggplotFill().reverse(true));
c.points().position(c.dodge2().width(.8).padding(.2).preserve('Single'));
c.points().position(c.jitterDodge(42).displacement(.2,.1));
c.points().position(c.nudge(.2,-.1));
c.points().textLabel('label');

const data=c.Data.columns({x:new Float64Array([1]),y:new Float64Array([2])});
c.errorbar().recipeValue('Lower',data.field('y')).recipeValue('Upper',c.sourceExpr(data.field('y')));
c.pointrange().recipeStatValue('Lower','Lower');
c.points().recipe({Raster:{interpolate:true}});
c.count().sumCount().x('x').y('y').countPartition(data.field('x'));
c.step('Mid'); c.abline(1,0); c.hline(0); c.vline(1);
c.bin().binWeight(c.sourceExpr(data.field('y')));
c.count().countWeight(c.sourceExpr(data.field('y')));

c.segment().lineend('Round').linejoin('Bevel');
c.crossbar().recipe({Interval:{kind:'Crossbar',fatten:3,middle:{linewidth:2}}});
c.boxplot().stat(c.boxplotStat().input(data.field('y')).weight(c.sourceExpr(data.field('x'))).width(.75));
c.violin().stat(c.violinStat().distributionOptions({Violin:{trim:true}}));
c.dotplot().stat(c.dotplotStat());
c.ecdf().stat(c.ecdfStat().univariateOptions({Ecdf:{n:10,pad:true}}));
c.qqLine().stat(c.qqLineStat());

c.density().recipe({Density:{outline:'Both'}}).stat(c.densityStat().input('x'));
c.qq().stat(c.qqStat());
c.functionCurve({Expression:{nodes:[{Read:'Value'}],output:0}});
c.functionStat({Expression:{nodes:[{Read:'Value'}],output:0}});
c.distributionStat({Boxplot:{coefficient:1.5}});
c.uniqueStat().x(data.field('x'));
c.alignStat().x('x').y('y');
c.connectStat('Hv').x('x').y('y');
c.univariateStat('Unique');
c.facetWrap('x').fields(['x','y']).columns(2).reference({drop:false,direction:'Tr',strip_position:'Left',axes:'All',axis_labels:'Margins'});
c.facetGrid('x','y').fields(['x','y']).rowFields(1).freeX(true).freeY(true).reference({space:'Free',margins:[0,1],shrink:false,switch:'Both',as_table:false});

c.smooth().stat(c.modelStat({method:'Linear',n:12}).modelOptions({method:'Linear'}).input('x').y('y').weight('w'));
c.quantile().stat(c.quantileStat()); c.smoothStat();
c.bin2d().stat(c.bin2dStat().spatialOptions({Rectangular:{axes:[{},{}],drop:true}}));
c.hex().stat(c.hexStat()); c.density2d().stat(c.density2dStat());
c.contour().stat(c.contourStat()); c.contourFilled().stat(c.contourFilledStat());
c.ellipse().stat(c.ellipseStat()); c.spatialStat({Ellipse:{level:.95,segments:51,kind:'Normal'}});
c.plot(data).coordinate({Cartesian:{flip:true}});

c.cutInterval([1,null,3],{n:2}).copy().column();
c.cutNumber([1,2,3],2).value();
c.cutWidth([1,2,3],1,{boundary:0}).dispose();
c.resolution([1,2],{zero:false});
c.summarize([1,null,3],{MeanSe:{mult:2}});
c.points().keyGlyph('example.diamond_key',1,{padding:.1});
const registry=c.ExtensionRegistry.example();
const materialized=registry.materialize('example.xy_recipe',1,{x:[1],y:[2]});
c.autoplot(materialized,'example.xy_recipe',1,{line:true},registry);
c.plot(data).withRegistry(registry).autolayer('example.xy_recipe',1,{line:true});

c.legend().scale("groups").registered("example.strip_guide",1,{});
c.facetWrap("group").registered("example.reverse_facets",1,{columns:2});

c.plot(data).registeredCoordinate("example.wave_coordinate",1,{amplitude:.1});

const saveOutput=new c.Output(new Uint8Array());
const saveOptions:c.SaveOptions={width:2,height:1,units:'in',dpi:'retina'};
const savePlan:c.SavePlan=saveOutput.resolve_save('plot-%03d.png',saveOptions,null,2);
saveOutput.save_figure(c.plot(data).layer(c.points()).build(),'plot.png',saveOptions,{device:frame=>frame.export('png')});
c.points().text_defaults();

const referenceTheme=c.theme().referencePreset('Minimal',{base_size:13}).fonts([]).updateElements({complete:false,elements:{'axis.text':'Blank'}}).replaceElements({complete:false,elements:{}});
const themeContext=new c.ThemeContext(referenceTheme);
const previousTheme:c.Theme=themeContext.set(themeContext.get());
previousTheme.dispose();themeContext.update({complete:false,elements:{}});themeContext.replace({complete:false,elements:{}});
c.plot(data).theme(themeContext.get());themeContext.dispose();
c.points().textDefaults();

function retainedPageDocuments(frame:c.FigureSnapshot):[Uint8Array,Uint8Array,Uint8Array] {
 const pages:c.FigurePages=frame.pages().append(frame);
 try {return [pages.export('pdf'),pages.export('ps'),pages.export('tiff')];}
 finally {pages.dispose();}
}
