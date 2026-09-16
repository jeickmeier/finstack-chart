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
d=data.frame(x=c(1,2,3,4),y=c(1,2,3,4),r=c('A','A','B','B'),c=c('L','R','L','R'))
p=ggplot(d,aes(x,y))+geom_point()
for(switch in list(NULL,'x','y','both'))capture(paste0('grid-switch-',if(is.null(switch))'none'else switch),p+facet_grid(r~c,switch=switch),list(switch=switch))
capture('grid-as-table-false',p+facet_grid(r~c,as.table=FALSE))
capture('partial-new-level',p+geom_point(data=data.frame(r='C',x=9,y=9))+facet_grid(r~c))
capture('numeric-facet',ggplot(transform(d,r=c(1.5,2.5,1.5,NA)),aes(x,y))+geom_point()+facet_wrap(vars(r)))
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/facet-edge-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null',null='null')
dev.off();cat('Captured',length(cases),'GG12 edge cases\n')
