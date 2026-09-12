# FIX-GG04: numeric callback arity interacts with each shared rescaler.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(spaced=c(1,3,9),all_missing=c(NA_real_,NA_real_),empty=numeric())
cases<-list()
for(kind in c('continuous','binned'))for(rescaler in c('range','maximum','midpoint'))for(population in names(sets))for(control in c('identity','reverse','fixed','lower_zero','missing_lower','empty','single')){
 inputs<-sets[[population]]
 fun<-function(x)switch(control,identity=x,reverse=rev(x),fixed=c(0,10),lower_zero=c(0,x[2]),missing_lower=c(NA_real_,x[2]),empty=numeric(),single=5)
 rescale<-switch(rescaler,range=scales::rescale,maximum=scales::rescale_max,midpoint=function(x,to=c(0,1),from=range(x,na.rm=TRUE))scales::rescale_mid(x,to,from,mid=2))
 result<-tryCatch(suppressWarnings({
  scale<-if(kind=='continuous')continuous_scale('size',palette=identity,limits=fun,rescaler=rescale)else binned_scale('size',palette=identity,limits=fun,rescaler=rescale)
  scale$train(inputs);limits<-scale$get_limits();values<-scale$map(inputs)
  guide<-tryCatch(list(breaks=encode(scale$get_breaks()),labels=as.list(unname(scale$get_labels()))),error=function(e)list(error=conditionMessage(e)))
  list(limits=encode(limits),values=encode(values),guide=guide)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,rescaler=rescaler,population=population,control=control,inputs=encode(inputs),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/limit-rescalers.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'numeric limit rescaler cases\n')
