# FIX-GG04: discrete positional functions select against the complete trained domain.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(x)lapply(unname(x),function(v)if(is.na(v))NULL else v)
cases<-list()
for(pop in c('ordinary','constant','missing','all_missing','empty'))
 for(limits in c('none','full')) for(drop in c(TRUE,FALSE)) for(translate in c(TRUE,FALSE))
  for(mode in c('domain','mixed','numeric','empty','null')) for(label_mode in c('indexed','automatic')) {
   values<-switch(pop,ordinary=c('b','a','c'),constant=c('a','a'),missing=c(NA,'b','c'),all_missing=c(NA_character_,NA_character_),empty=character());calls<-list();label_calls<-list()
   breaks<-function(x) {
    calls[[length(calls)+1L]]<<-list(values=encode(x),names=as.list(names(x)))
    switch(mode,domain=x,mixed=setNames(c('outside','a','a',NA,'c'),c('off','first','duplicate','missing','last')),numeric=setNames(c(1,NA,1),c('one','missing','again')),empty=character(),null=NULL)
   }
   labels<-function(x) {label_calls[[length(label_calls)+1L]]<<-list(values=encode(x),names=as.list(names(x)));paste0(seq_along(x),'/',length(x))}
   result<-tryCatch(suppressWarnings({
    s<-scale_x_discrete(limits=if(limits=='full')c('c','b','a',NA)else NULL,drop=drop,na.translate=translate,breaks=breaks,labels=if(label_mode=='indexed')labels else waiver())
    b<-ggplot_build(ggplot(data.frame(v=factor(values,levels=c('b','a','c','unused')),y=rep(1,length(values))),aes(v,y))+geom_point()+s)
    panel<-b$layout$panel_params[[1]]$x
    list(values=encode(panel$get_breaks()),labels=encode(panel$get_labels()),range=encode(panel$continuous_range))
   }),error=function(e)list(error=conditionMessage(e)))
   cases[[length(cases)+1L]]<-list(population=pop,limits=limits,drop=drop,translate=translate,mode=mode,label_mode=label_mode,inputs=encode(values),calls=calls,label_calls=label_calls,result=result)
  }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-discrete-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete positional break function builds\n')
