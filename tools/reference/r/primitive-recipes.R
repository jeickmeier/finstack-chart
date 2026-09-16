# GG07: pinned built data and actual drawing outcomes; no implementation acceptance.
pdf(file=tempfile(fileext='.pdf'),width=7,height=5)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
library(grid)
unit_record=function(x)list(value=as.numeric(x),unit=as.character(x))
grob_record=function(g,depth=0L) {
 if(depth>40L)stop('grob recursion budget exceeded')
 result=list(class=class(g)[1],name=g$name)
 for(n in intersect(names(g),c('x','y','x0','x1','x2','y0','y1','y2','width','height','r','id','id.lengths','pathId','pathId.lengths','rule','pch','size','curvature','angle','ncp','shape','open','ends'))) {
  v=g[[n]];if(inherits(v,'unit'))result[[n]]=unit_record(v) else if(is.atomic(v))result[[n]]=unname(v)
 }
 if(!is.null(g$gp))result$gp=unclass(g$gp)
 if(!is.null(g$arrow))result$arrow=list(angle=g$arrow$angle,length=unit_record(g$arrow$length),ends=g$arrow$ends,type=g$arrow$type)
 if(!is.null(g$raster))result$raster=list(dim=dim(g$raster),pixels=as.vector(g$raster),interpolate=g$interpolate)
 if(!is.null(g$children))result$children=lapply(g$children,function(c)grob_record(c,depth+1L))
 if(!is.null(g$grobs))result$grobs=lapply(g$grobs,function(c)grob_record(c,depth+1L))
 result
}
cases=list()
capture=function(name,p,controls=list()) {
 warnings=character();messages=character();entry=list(name=name,controls=controls)
 result=withCallingHandlers(tryCatch({
  b=ggplot_build(p);entry$data=b$data
  entry$draw=tryCatch({gt=ggplot_gtable(b);grid.newpage();grid.draw(gt);panel=grep('^panel',gt$layout$name);list(ok=TRUE,panels=lapply(gt$grobs[panel],grob_record))},error=function(e)list(ok=FALSE,error=conditionMessage(e)))
  entry
 },error=function(e){entry$build_error=conditionMessage(e);entry}),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')},message=function(m){messages<<-c(messages,conditionMessage(m));invokeRestart('muffleMessage')})
 result$warnings=warnings;result$messages=messages;cases[[length(cases)+1L]]<<-result
}
d=data.frame(x=c(1,2,3),y=c(2,-1,0),lo=c(1,-2,0),hi=c(3,0,0))
for(geom in c('linerange','pointrange','crossbar','errorbar'))for(horizontal in c(FALSE,TRUE)) {
 mapping=if(horizontal)aes(y=x,x=y,xmin=lo,xmax=hi)else aes(x=x,y=y,ymin=lo,ymax=hi)
 constructor=get(paste0('geom_',geom));args=list(mapping=mapping,orientation=if(horizontal)'y'else'x')
 if(geom %in% c('crossbar','errorbar'))args$width=.4
 capture(paste(geom,if(horizontal)'horizontal'else'vertical',sep='-'),ggplot(d)+do.call(constructor,args),list(geom=geom,horizontal=horizontal,width=if(geom %in% c('crossbar','errorbar')).4 else NULL))
}
summary_data=data.frame(x=rep(1:2,each=3),y=c(1,3,5,-2,0,4))
capture('summary-pointrange',ggplot(summary_data,aes(x,y))+stat_summary(fun=mean,fun.min=min,fun.max=max,geom='pointrange'))
base=ggplot(data.frame(x=c(1,4),y=c(-1,4)),aes(x,y))+geom_blank()
for(log in c(FALSE,TRUE)) {
 p=base+geom_abline(slope=c(.5,-1),intercept=c(1,3))+geom_hline(yintercept=c(0,2))+geom_vline(xintercept=c(1,3))
 if(log)p=p+scale_x_log10()
 capture(paste0('reference-lines-',if(log)'log'else'linear'),p,list(log=log))
}
for(direction in c('hv','vh','mid'))for(gap in c(FALSE,TRUE)) {
 d=data.frame(x=c(1,3,2,4),y=c(2,1,3,4));if(gap)d$y[3]=NA
 capture(paste('step',direction,gap,sep='-'),ggplot(d,aes(x,y))+geom_step(direction=direction),list(direction=direction,gap=gap))
}
d=data.frame(x=c(1,2),y=c(1,3),xend=c(3,4),yend=c(2,0))
for(ends in c('first','last','both'))for(type in c('open','closed'))capture(paste('segment',ends,type,sep='-'),ggplot(d,aes(x,y,xend=xend,yend=yend))+geom_segment(arrow=arrow(ends=ends,type=type,length=unit(3,'mm'))),list(ends=ends,type=type,length_mm=3))
for(curvature in c(-.4,.4))capture(paste0('curve-',curvature),ggplot(d,aes(x,y,xend=xend,yend=yend))+geom_curve(curvature=curvature,angle=60,ncp=5,arrow=arrow(type='closed',length=unit(3,'mm'))),list(curvature=curvature,angle=60,ncp=5))
spokes=data.frame(x=c(1,2,3),y=c(1,1,2),angle=c(0,pi/2,pi),radius=c(1,-1,2))
capture('spokes-signed',ggplot(spokes,aes(x,y,angle=angle,radius=radius))+geom_spoke(),list(angles=spokes$angle,radii=spokes$radius))
for(rule in c('evenodd','winding'))for(reverse in c(FALSE,TRUE)) {
 outer=data.frame(x=c(0,4,4,0),y=c(0,0,4,4),subgroup=1L)
 inner=data.frame(x=c(1,3,3,1),y=c(1,1,3,3),subgroup=2L);if(reverse)inner=inner[4:1,]
 d=rbind(outer,inner);d$group=1L
 capture(paste('polygon-hole',rule,reverse,sep='-'),ggplot(d,aes(x,y,group=group,subgroup=subgroup))+geom_polygon(rule=rule),list(rule=rule,reverse=reverse))
}
grid_data=expand.grid(x=c(1,2,3),y=c(1,2));grid_data$value=seq_len(nrow(grid_data))
for(explicit in c(FALSE,TRUE))capture(paste0('tile-',explicit),ggplot(grid_data,aes(x,y,fill=value))+if(explicit)geom_tile(width=.8,height=.6)else geom_tile(),list(explicit=explicit))
for(interpolate in c(FALSE,TRUE))for(missing in c(FALSE,TRUE)) {
 d=if(missing)grid_data[-2,]else grid_data
 capture(paste('raster',interpolate,missing,sep='-'),ggplot(d,aes(x,y,fill=value))+geom_raster(interpolate=interpolate),list(interpolate=interpolate,missing=missing))
}
capture('raster-transformed',ggplot(grid_data,aes(x,y,fill=value))+geom_raster()+coord_polar())
for(sides in c('bl','tr','bltr'))capture(paste0('rug-',sides),ggplot(data.frame(x=c(0,1,2),y=c(0,2,1)),aes(x,y))+geom_rug(sides=sides,length=unit(.04,'npc')),list(sides=sides,length_npc=.04))
capture('blank-extrema',ggplot(data.frame(x=c(-10,20),y=c(-5,30)),aes(x,y))+geom_blank())
count_data=data.frame(x=c(1,1,2,2,2),y=c(1,1,1,2,2),weight=c(1,2,1,0,3))
capture('weighted-count',ggplot(count_data,aes(x,y,weight=weight))+geom_count())
capture('unweighted-count',ggplot(count_data,aes(x,y))+geom_count())
capture('polygon-invalid-fill-rule',ggplot(data.frame(x=c(0,1,0),y=c(0,0,1)),aes(x,y))+geom_polygon(rule='invalid'),list(rule='invalid'))
capture('curve-nonfinite-curvature',ggplot(data.frame(x=0,y=0,xend=1,yend=1),aes(x,y,xend=xend,yend=yend))+geom_curve(curvature=Inf),list(curvature='Infinity'))
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',device=list(kind='pdf',width_inches=7,height_inches=5),cases=cases),'fixtures/parity/ggplot2/primitive-recipes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null',null='null')
