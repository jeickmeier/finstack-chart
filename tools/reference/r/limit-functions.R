# FIX-GG04: pure dynamic limits, including raw training space and missing endpoints.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(spaced=c(1,3,9),constant=c(4,4),missing=c(NA,1,9),all_missing=c(NA_real_,NA_real_),empty=numeric(),invalid=c(-9,-1,0,4))
cases<-list()
for(kind in c('continuous','binned','identity'))for(transform in c('identity','sqrt','reverse'))for(population in names(sets))for(control in c('identity','reverse','fixed','lower_zero','missing_lower','empty','single')){
 inputs<-sets[[population]];seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-encode(x);switch(control,identity=x,reverse=rev(x),fixed=c(0,10),lower_zero=c(0,x[2]),missing_lower=c(NA_real_,x[2]),empty=numeric(),single=5)}
 result<-tryCatch(suppressWarnings({
  scale<-switch(kind,continuous=scale_size_continuous(limits=fun,transform=transform),binned=scale_size_binned(limits=fun,transform=transform),identity=scale_alpha_identity(limits=fun,transform=transform,guide='legend'))
  if(is.null(scale$palette))scale$palette<-scale$fallback_palette
  scale$train(scale$transform(inputs));limits<-scale$get_limits();values<-scale$map(scale$transform(inputs))
  guide<-tryCatch(list(breaks=encode(scale$get_breaks()),labels=as.list(unname(scale$get_labels()))),error=function(e)list(error=conditionMessage(e)))
  list(limits=encode(limits),values=encode(values),guide=guide)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,transform=transform,population=population,control=control,inputs=encode(inputs),seen=seen,result=result)
}
keys<-list(spaced=c('b','a','c'),constant=c('b','b'),missing=c(NA,'b','a'),all_missing=c(NA_character_,NA_character_),empty=character())
for(kind in c('discrete','identity'))for(guide_enabled in if(kind=='identity')c(TRUE,FALSE)else TRUE)for(population in names(keys))for(control in c('identity','reverse','fixed','append','empty','numeric')){
 inputs<-keys[[population]];seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-encode(x);switch(control,identity=x,reverse=rev(x),fixed=c('c','a'),append=c(x,'extra'),empty=character(),numeric=c(3,1))}
 result<-tryCatch(suppressWarnings({
  scale<-if(kind=='discrete')scale_shape_discrete(limits=fun)else scale_linetype_identity(limits=fun,guide=if(guide_enabled)'legend'else'none')
  if(is.null(scale$palette))scale$palette<-scale$fallback_palette
  scale$train(inputs);limits<-scale$get_limits();values<-scale$map(inputs)
  guide<-tryCatch(list(breaks=encode(scale$get_breaks()),labels=as.list(unname(scale$get_labels()))),error=function(e)list(error=conditionMessage(e)))
  list(limits=encode(limits),values=encode(values),guide=guide)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,guide=guide_enabled,population=population,control=control,inputs=encode(inputs),seen=seen,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/limit-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'limit-function reference cases\n')
