# FIX-GG04: exact binned color-label inputs after cut selection.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
cases<-list()
for(tr in c('identity','sqrt','log10','reverse'))
 for(pop in c('ordinary','constant','missing','all_missing','empty'))
  for(limits in c('none','full'))
   for(bm in c('nice','equal','explicit','empty'))
    for(mode in c('indexed','missing','short','empty')) {
     values<-switch(pop,ordinary=c(1,4,10),constant=c(4,4),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric());calls<-list()
     label_function<-function(x) {
      calls[[length(calls)+1L]]<<-list(values=encode(x),names=as.list(names(x)))
      labels<-paste0(seq_along(x),'/',length(x))
      switch(mode,indexed=labels,missing=replace(labels,seq_along(x)%%2==0,NA_character_),short=head(labels,1),empty=character())
     }
     make_scale<-function()scale_colour_steps(transform=tr,limits=if(limits=='full')c(1,10)else NULL,n.breaks=5,nice.breaks=bm!='equal',breaks=if(bm=='explicit')c(-1,0,1,1,3,20,Inf,NA)else if(bm=='empty')numeric()else waiver(),labels=label_function)
     direct<-tryCatch(suppressWarnings({
      s<-make_scale();s$train(s$transform(values));breaks<-s$get_breaks();labels<-s$get_labels(breaks)
      list(breaks=encode(breaks),labels=encode(labels),limits=encode(s$get_limits()))
     }),error=function(e)list(error=conditionMessage(e)))
     direct_calls<-calls;calls<-list()
     result<-tryCatch(suppressWarnings({
      s<-make_scale()
      b<-ggplot_build(ggplot(data.frame(x=seq_along(values),v=values),aes(x,1,colour=v))+geom_point()+s)
      keys<-lapply(b$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label)))
      list(keys=unname(keys))
     }),error=function(e)list(error=conditionMessage(e)))
     cases[[length(cases)+1L]]<-list(transform=tr,population=pop,limits=limits,break_mode=bm,label_mode=mode,inputs=encode(values),direct_calls=direct_calls,direct=direct,calls=calls,result=result)
    }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/binned-label-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'binned label callback records\n')
