# FIX-GG04: nonempty nonfinite training is not an empty scale or a [0,1] guide.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases<-list()
encode<-function(v)lapply(v,function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity' else '-Infinity' else unname(x))
for(family in c('continuous','identity'))for(population in c('missing','nonfinite'))for(tr in c('identity','sqrt','log10','reverse'))for(lm in c('none','lower','upper','full'))for(bm in c('auto','explicit','empty')) {
 limits<-switch(lm,none=NULL,lower=c(1,NA),upper=c(NA,10),full=c(1,10))
 breaks<-switch(bm,auto=waiver(),explicit=c(0,1,2,20),empty=numeric())
 s<-if(family=='identity')scale_size_identity(transform=tr,limits=limits,breaks=breaks,guide='legend')else scale_colour_gradient(transform=tr,limits=limits,breaks=breaks)
 values<-if(population=='missing')c(NA_real_,NA_real_)else c(Inf,-Inf)
 s$train(suppressWarnings(s$transform(values)))
 result<-tryCatch({b<-s$get_breaks();list(breaks=encode(b),labels=as.list(unname(s$get_labels(b))),visible=as.list(is.finite(scales::oob_censor_any(b,s$get_limits()))))},error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(family=family,population=population,transform=tr,limits=lm,breaks=bm,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/nonfinite-continuous-guides.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'nonfinite continuous/identity guide records\n')
