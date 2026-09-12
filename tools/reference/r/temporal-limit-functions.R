# FIX-GG04: Date/datetime function limits preserve class and source coordinates.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(as.numeric(v)),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(spaced=c(0,1,4,9),constant=c(4,4),missing=c(NA,0,9),all_missing=c(NA_real_,NA_real_),empty=numeric(),fractional=c(0.125,1.5,4.75))
cases<-list()
for(kind in c('date','datetime'))for(population in names(sets))for(control in c('identity','reverse','fixed','lower_zero','missing_lower','empty','single')){
 convert<-function(v)if(kind=='date')as.Date(v+19723,origin='1970-01-01')else as.POSIXct(v+1704067200,origin='1970-01-01',tz='UTC')
 inputs<-convert(sets[[population]]);seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-list(class=as.list(class(x)),values=encode(x));switch(control,identity=x,reverse=rev(x),fixed=convert(c(0,10)),lower_zero=c(convert(0),x[2]),missing_lower=c(convert(NA_real_),x[2]),empty=convert(numeric()),single=convert(5))}
 result<-tryCatch(suppressWarnings({
  scale<-if(kind=='date')scale_size_date(limits=fun)else scale_size_datetime(limits=fun)
  if(is.null(scale$palette))scale$palette<-scale$fallback_palette
  scale$train(scale$transform(inputs));limits<-scale$get_limits();values<-scale$map(scale$transform(inputs))
  guide<-tryCatch(list(breaks=encode(scale$get_breaks()),labels=as.list(unname(scale$get_labels()))),error=function(e)list(error=conditionMessage(e)))
  list(limits=encode(limits),values=encode(values),guide=guide)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,population=population,control=control,inputs=encode(inputs),seen=seen,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/temporal-limit-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'temporal limit-function reference cases\n')
