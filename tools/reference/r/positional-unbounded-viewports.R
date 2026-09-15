# FIX-GG04: finite coordinate views over unbounded position-scale populations.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(as.numeric(v)),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(finite=c(1,4,9),infinite=c(-Inf,1,9,Inf),missing=c(NA_real_,1,9),empty=numeric())
bounds<-list(both=c(-Inf,Inf),lower=c(-Inf,10),upper=c(0,Inf),positive=c(Inf,Inf),negative=c(-Inf,-Inf),finite=c(0,10))
views<-list(full=c(0,10),positive=c(1,5),reverse=c(10,0))
cases<-list()
for(kind in c('continuous','binned'))for(transform in c('identity','sqrt','reverse','log10'))for(population in names(sets))for(control in names(bounds))for(view in names(views))for(oob in c('censor','squish','keep')){
 inputs<-sets[[population]];limit<-bounds[[control]];viewport<-views[[view]]
 result<-tryCatch(suppressWarnings({
  constructor<-if(kind=='binned')scale_x_binned else scale_x_continuous
  p<-ggplot(data.frame(x=inputs,y=rep(1,length(inputs))),aes(x,y))+geom_point()+do.call(constructor,list(limits=limit,transform=transform,oob=switch(oob,censor=scales::oob_censor,squish=scales::oob_squish,keep=scales::oob_keep)))+coord_cartesian(xlim=viewport)
  b<-ggplot_build(p);panel<-b$layout$panel_params[[1]];rendered<-b$layout$coord$transform(b$data[[1]],panel)
  list(values=encode(b$data[[1]]$x),point_positions=encode(rendered$x),limits=encode(panel$x$limits),range=encode(panel$x.range),breaks=encode(panel$x$breaks),positions=encode(panel$x$break_positions()),labels=as.list(unname(panel$x$get_labels())))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,transform=transform,population=population,control=control,view=view,oob=oob,inputs=encode(inputs),limits=encode(limit),viewport=encode(viewport),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-unbounded-viewports.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'unbounded viewport builds\n')
