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
