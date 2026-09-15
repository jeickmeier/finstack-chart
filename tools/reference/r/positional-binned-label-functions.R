# FIX-GG04: exact binned primary-axis label inputs after cut selection.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
cases<-list()
for(tr in c('identity','sqrt','log10','reverse'))
 for(pop in c('ordinary','constant','missing','all_missing','empty'))
  for(limits in c('none','full'))
   for(show_limits in c(FALSE,TRUE))
   for(bm in c('nice','equal','explicit','empty'))
    for(mode in c('indexed','missing','short','empty','default')) {
     values<-switch(pop,ordinary=c(1,4,10),constant=c(4,4),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric());calls<-list()
     label_function<-function(x) {
      calls[[length(calls)+1L]]<<-list(values=encode(x),names=as.list(names(x)))
      labels<-paste0(seq_along(x),'/',length(x))
      switch(mode,indexed=labels,missing=replace(labels,seq_along(x)%%2==0,NA_character_),short=head(labels,1),empty=character())
     }
     make_scale<-function()scale_x_binned(show.limits=show_limits,transform=tr,limits=if(limits=='full')c(1,10)else NULL,n.breaks=5,nice.breaks=bm!='equal',breaks=if(bm=='explicit')c(-1,0,1,1,3,20,Inf,NA)else if(bm=='empty')numeric()else waiver(),labels=if(mode=='default')waiver()else label_function)
     result<-tryCatch(suppressWarnings({
      s<-make_scale()
      b<-ggplot_build(ggplot(data.frame(v=values,y=rep(1,length(values))),aes(v,y))+geom_point()+s)
      panel<-b$layout$panel_params[[1]]$x
      list(values=encode(panel$get_breaks()),labels=encode(panel$get_labels()),range=encode(panel$continuous_range))
     }),error=function(e)list(error=conditionMessage(e)))
     cases[[length(cases)+1L]]<-list(transform=tr,population=pop,limits=limits,show_limits=show_limits,break_mode=bm,label_mode=mode,inputs=encode(values),calls=calls,result=result)
    }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-binned-label-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'binned label callback records\n')
