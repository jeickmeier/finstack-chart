stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile('gg05-steps-',fileext='.pdf'))
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0) 'Infinity' else '-Infinity' else v)
cases=list()
for(family in c('continuous','binned')) for(endpoints in c(FALSE,TRUE)) for(even in c(FALSE,TRUE)) for(show_limits in c(FALSE,TRUE)) for(direction in c('vertical','horizontal')) for(reverse in c(FALSE,TRUE)) {
 warnings=character()
 result=tryCatch(withCallingHandlers({
 args=list(colours=c('#0000ff','#ffffff','#ff0000'),values=c(0,.2,1),limits=c(-2,8),breaks=if(endpoints)c(-2,0,3,8)else c(0,3),guide=guide_coloursteps(even.steps=even,show.limits=show_limits,direction=direction,reverse=reverse))
 scale=do.call(if(family=='binned')scale_colour_stepsn else scale_colour_gradientn,args)
 p=ggplot(data.frame(x=1:4,v=c(-2,0,3,8)),aes(x,1,colour=v))+geom_point()+scale
 b=ggplot_build(p);params=b$plot$guides$params[[1]];gt=ggplot_gtable(b);box=gt$grobs[[which(gt$layout$name=='guide-box-right')]];guide=box$grobs[[1]];bar=guide$grobs[[which(guide$layout$name=='bar')]]
 list(mapped=encode(b$data[[1]]$colour),decor=lapply(params$decor,encode),key=lapply(params$key,encode),bar=list(class=as.list(class(bar)),x=encode(as.numeric(bar$x)),y=encode(as.numeric(bar$y)),width=encode(as.numeric(bar$width)),height=encode(as.numeric(bar$height)),colors=encode(bar$gp$fill)))
 },warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]=list(family=family,palette='asymmetric',endpoints=endpoints,even_steps=even,show_limits=show_limits,direction=direction,reverse=reverse,warnings=as.list(warnings),result=result)
}
for(family in 'binned') for(endpoints in c(FALSE,TRUE)) for(even in TRUE) for(show_limits in FALSE) for(direction in 'vertical') for(reverse in FALSE) {
 warnings=character()
 result=tryCatch(withCallingHandlers({
 args=list(colours=c('#0000ff','#ffffff','#ff0000'),values=c(0,.2,1),limits=c(-2,8),breaks=if(endpoints)c(-2,0,3,8)else c(0,3))
 scale=do.call(if(family=='binned')scale_colour_stepsn else scale_colour_gradientn,args)
 p=ggplot(data.frame(x=1:4,v=c(-2,0,3,8)),aes(x,1,colour=v))+geom_point()+scale
 b=ggplot_build(p);params=b$plot$guides$params[[1]];gt=ggplot_gtable(b);box=gt$grobs[[which(gt$layout$name=='guide-box-right')]];guide=box$grobs[[1]];bar=guide$grobs[[which(guide$layout$name=='bar')]]
 list(mapped=encode(b$data[[1]]$colour),decor=lapply(params$decor,encode),key=lapply(params$key,encode),bar=list(class=as.list(class(bar)),x=encode(as.numeric(bar$x)),y=encode(as.numeric(bar$y)),width=encode(as.numeric(bar$width)),height=encode(as.numeric(bar$height)),colors=encode(bar$gp$fill)))
 },warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]=list(guide_kind='default',family=family,palette='asymmetric',endpoints=endpoints,even_steps=even,show_limits=show_limits,direction=direction,reverse=reverse,warnings=as.list(warnings),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/colorsteps-layout.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
invisible(dev.off())
cat(length(cases),'colorsteps layout cases\n')
