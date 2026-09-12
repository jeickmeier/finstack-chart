# FIX-GG04: positional limit functions participate before and after statistics.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(spaced=c(1,3,9),constant=c(4,4),missing=c(NA,1,9),all_missing=c(NA_real_,NA_real_),empty=numeric(),invalid=c(-9,-1,0,4))
cases<-list()
for(kind in c('continuous','binned'))for(transform in c('identity','sqrt','reverse'))for(population in names(sets))for(control in c('identity','reverse','fixed','lower_zero','missing_lower','empty','single')){
 inputs<-sets[[population]];seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-encode(x);switch(control,identity=x,reverse=rev(x),fixed=c(0,10),lower_zero=c(0,x[2]),missing_lower=c(NA_real_,x[2]),empty=numeric(),single=5)}
 result<-tryCatch(suppressWarnings({
  scale<-if(kind=='continuous')scale_x_continuous(limits=fun,transform=transform)else scale_x_binned(limits=fun,transform=transform)
  b<-ggplot_build(ggplot(data.frame(x=inputs,y=rep(1,length(inputs))),aes(x,y))+geom_point()+scale)
  s<-b$layout$panel_scales_x[[1]];panel<-b$layout$panel_params[[1]]
  list(values=encode(b$data[[1]]$x),limits=encode(s$get_limits()),range=encode(panel$x.range),breaks=encode(panel$x$breaks),labels=as.list(unname(panel$x$get_labels())))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,transform=transform,population=population,control=control,inputs=encode(inputs),seen=seen,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-limit-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'positional limit-function reference builds\n')
