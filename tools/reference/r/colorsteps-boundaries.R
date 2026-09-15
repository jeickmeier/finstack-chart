stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile('gg05-step-boundaries-',fileext='.pdf'))
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0) 'Infinity' else '-Infinity' else v)
capture <- function(p) {
 b=ggplot_build(p);gt=ggplot_gtable(b)
 params=if(length(b$plot$guides$params))b$plot$guides$params[[1]]else NULL
 box=gt$grobs[[which(gt$layout$name=='guide-box-right')]]
 guide=if(length(box$grobs))box$grobs[[1]]else NULL
 bar=if(!is.null(guide)&&'bar'%in%guide$layout$name)guide$grobs[[which(guide$layout$name=='bar')]]else NULL
 list(mapped=encode(b$data[[1]]$colour),decor=if(is.null(params$decor))NULL else lapply(params$decor,encode),key=if(is.null(params$key))NULL else lapply(params$key,encode),bar=if(is.null(bar))NULL else list(class=as.list(class(bar)),x=encode(as.numeric(bar$x)),y=encode(as.numeric(bar$y)),width=encode(as.numeric(bar$width)),height=encode(as.numeric(bar$height)),colors=encode(bar$gp$fill)))
}
defaults='--default' %in% commandArgs(trailingOnly=TRUE)
cases=list()
for(family in if(defaults)'binned'else c('continuous','binned')) for(population in c('ordinary','constant','empty')) for(mode in c('automatic','null','empty','one','duplicates','outside','nonfinite','unsorted','limits','endpoint_first',if(defaults)'mixed_edges')) {
 warnings=character();limits=if(population=='constant')c(2,2)else c(-2,8);values=switch(population,ordinary=c(-2,0,3,8),constant=rep(2,4),empty=numeric())
 breaks=switch(mode,automatic=waiver(),null=NULL,empty=numeric(),one=if(population=='constant')2 else 0,duplicates=c(0,0,3),outside=c(-4,0,3,10),nonfinite=c(NA_real_,Inf,-Inf,0,3),unsorted=c(8,3,0,-2),limits=c(-2,8),endpoint_first=c(8,0,3),mixed_edges=c(-Inf,-4,0,3,10,Inf))
 result=tryCatch(withCallingHandlers({
 args=list(colours=c('#0000ff','#ffffff','#ff0000'),values=c(0,.2,1),limits=limits,breaks=breaks,guide=guide_coloursteps())
 if(defaults)args$guide=NULL
 scale=do.call(if(family=='binned')scale_colour_stepsn else scale_colour_gradientn,args)
 capture(ggplot(data.frame(x=seq_along(values),v=values),aes(x,1,colour=v))+geom_point()+scale)
 },warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]=list(family=family,population=population,mode=mode,limits=encode(limits),inputs=encode(values),breaks=if(mode=='automatic')'automatic'else if(mode=='null')NULL else encode(breaks),warnings=as.list(warnings),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),if(defaults)'fixtures/parity/ggplot2/colorsteps-default-boundaries.json'else 'fixtures/parity/ggplot2/colorsteps-boundaries.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
invisible(dev.off())
cat(length(cases),'colorsteps boundary cases\n')
