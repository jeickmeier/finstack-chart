# FIX-GG04: empty and all-nonfinite binned scale training, before guide composition.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases<-list()
encode<-function(v)lapply(v,function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity' else '-Infinity' else unname(x))
for(pop in c('empty','missing','infinite'))for(tr in c('identity','sqrt','log10','reverse'))for(lm in c('none','lower','upper','full'))for(bm in c('nice','equal','explicit','empty')) {
 result<-tryCatch({
  s<-scale_colour_steps(transform=tr,limits=switch(lm,none=NULL,lower=c(1,NA),upper=c(NA,10),full=c(1,10)),breaks=switch(bm,nice=waiver(),equal=waiver(),explicit=c(1,2),empty=numeric()),nice.breaks=bm!='equal',n.breaks=5)
  x<-switch(pop,empty=numeric(),missing=c(NA_real_,NA_real_),infinite=c(Inf,-Inf));s$train(suppressWarnings(s$transform(x)))
  b<-suppressWarnings(s$get_breaks());list(breaks=encode(b),labels=as.list(unname(s$get_labels(b))),visible=as.list(is.finite(scales::oob_censor_any(b,s$get_limits()))))
 },error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(population=pop,transform=tr,limits=lm,breaks=bm,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/binned-populations.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'binned population records\n')
