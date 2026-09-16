# GG12 pinned facet training/layout/draw fixtures, independent of the Rust engine.
library(ggplot2)
library(grid)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
pdf(file=tempfile(fileext='.pdf'),width=8,height=6)
labels=function(g){out=character();if(inherits(g,'text'))out=c(out,as.character(g$label));if(!is.null(g$children))for(v in g$children)out=c(out,labels(v));if(!is.null(g$grobs))for(v in g$grobs)out=c(out,labels(v));out}
units=function(x)list(value=as.numeric(x),description=as.character(x))
cases=list()
capture=function(name,p,controls=list()) {
 warnings=character();messages=character();entry=list(name=name,controls=controls)
 withCallingHandlers(tryCatch({
 b=ggplot_build(p);entry$layout=b$layout$layout;entry$data=b$data
 entry$ranges=lapply(b$layout$panel_params,function(p)list(x=p$x.range,y=p$y.range,x_breaks=p$x$get_breaks(),y_breaks=p$y$get_breaks()))
 entry$draw=tryCatch({g=ggplot_gtable(b);grid.newpage();grid.draw(g);index=grep('^(panel|strip|axis)',g$layout$name);list(ok=TRUE,layout=g$layout[index,],widths=units(g$widths),heights=units(g$heights),labels=lapply(g$grobs[index],labels))},error=function(e)list(ok=FALSE,error=conditionMessage(e)))
 entry$build=list(ok=TRUE)
 },error=function(e){entry$build<<-list(ok=FALSE,error=conditionMessage(e))}),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')},message=function(m){messages<<-c(messages,conditionMessage(m));invokeRestart('muffleMessage')})
 entry$warnings=warnings;entry$messages=messages;cases[[length(cases)+1L]]<<-entry
}
d=data.frame(x=c(1,2,1,4,1,10,2),y=c(1,3,2,8,10,100,4),r=factor(c('B','B','A','A','B','B',NA),levels=c('B','A','unused')),nested=c('u','u','v','v','w','w','u'),c=factor(c('L','L','R','R','R','R','L'),levels=c('R','L','unused')),z=c('z1','z2','z1','z2','z1','z2','z1'))
p=ggplot(d,aes(x,y))+geom_point()
for(drop in c(TRUE,FALSE)) {
 capture(paste0('wrap-multiple-drop-',drop),p+facet_wrap(vars(r,nested),drop=drop),list(drop=drop))
 capture(paste0('grid-nested-crossed-drop-',drop),p+facet_grid(rows=vars(r,nested),cols=vars(c,z),drop=drop),list(drop=drop))
}
for(margin in list(TRUE,'r',c('r','nested'))) capture(paste0('margins-',paste(margin,collapse='-')),ggplot(d,aes(x))+stat_count()+facet_grid(rows=vars(r,nested),cols=vars(c),margins=margin),list(margins=margin))
for(shrink in c(TRUE,FALSE))capture(paste0('summary-shrink-',shrink),ggplot(d,aes(x=1,y=y))+stat_summary(fun=mean,geom='point')+facet_wrap(vars(c),scales='free_y',shrink=shrink),list(shrink=shrink))
for(scales in c('fixed','free_x','free_y','free'))for(space in c('fixed','free')) capture(paste0('grid-',scales,'-space-',space),p+facet_grid(r~c,scales=scales,space=space),list(scales=scales,space=space))
for(direction in c('h','v','tr','rt','bl','lb'))capture(paste0('wrap-direction-',direction),p+facet_wrap(vars(r,c),ncol=2,dir=direction),list(direction=direction))
for(position in c('top','bottom','left','right'))capture(paste0('strips-',position),p+facet_wrap(vars(r,c),strip.position=position),list(strip_position=position))
for(axes in c('margins','all_x','all_y','all'))capture(paste0('axes-',axes),p+facet_wrap(vars(r,c),axes=axes,axis.labels='margins'),list(axes=axes,axis_labels='margins'))
capture('missing-facet-broadcast',p+geom_point(data=data.frame(x=3,y=4),colour='red')+facet_grid(r~c))
capture('missing-one-facet-variable',p+geom_point(data=data.frame(x=3,y=4,r=factor('B',levels=levels(d$r))),colour='red')+facet_grid(r~c))
capture('label-both',p+facet_grid(r~c,labeller=label_both))
capture('label-context',p+facet_grid(r~c,labeller=labeller(r=c(B='Row B',A='Row A'),c=label_both)))
long=transform(d,r=paste('Long facet heading with words',r))
capture('label-wrap',ggplot(long,aes(x,y))+geom_point()+facet_wrap(vars(r),labeller=label_wrap_gen(width=12)))
capture('grid-free-axis-labels',p+facet_grid(r~c,scales='free',axes='all',axis.labels='all'))
capture('wrap-free-space-direction',p+facet_wrap(vars(r,c),scales='free_x',space='free_x',ncol=2),list(space='free_x',ncol=2))
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',device=list(width_inches=8,height_inches=6),cases=cases),'fixtures/parity/ggplot2/facet-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null',null='null')
dev.off();cat('Captured',length(cases),'GG12 facet cases\n')
