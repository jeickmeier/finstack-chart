# FIX-GG04: discrete primary-axis labels after break matching and missing-level policy.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else v)
cases<-list()
for(pop in c('ordinary','constant','missing','all_missing','empty'))
 for(limits in c('none','full'))
  for(drop in c(TRUE,FALSE))
   for(translate in c(TRUE,FALSE))
    for(bm in c('automatic','explicit','empty','named'))
     for(mode in c('indexed','missing','short','empty')) {
      values<-switch(pop,ordinary=c('b','a','c'),constant=c('a','a'),missing=c(NA,'b','c'),all_missing=c(NA_character_,NA_character_),empty=character());calls<-list()
      label_function<-function(x) {
       calls[[length(calls)+1L]]<<-list(values=encode(x),names=as.list(names(x)))
       labels<-paste0(seq_along(x),'/',length(x))
       switch(mode,indexed=labels,missing=replace(labels,seq_along(x)%%2==0,NA_character_),short=head(labels,1),empty=character())
      }
      result<-tryCatch(suppressWarnings({
       s<-scale_x_discrete(limits=if(limits=='full')c('c','b','a',NA)else NULL,drop=drop,na.translate=translate,breaks=if(bm=='explicit')c('outside','a','a',NA,'c')else if(bm=='empty')character()else if(bm=='named')setNames(c('outside','a','a',NA,'c'),c('off','first','duplicate','missing','last'))else waiver(),labels=label_function)
       b<-ggplot_build(ggplot(data.frame(v=factor(values,levels=c('b','a','c','unused')),y=rep(1,length(values))),aes(v,y))+geom_point()+s)
       panel<-b$layout$panel_params[[1]]$x
       list(values=encode(panel$get_breaks()),labels=encode(panel$get_labels()),range=encode(panel$continuous_range))
      }),error=function(e)list(error=conditionMessage(e)))
      cases[[length(cases)+1L]]<-list(population=pop,limits=limits,drop=drop,translate=translate,break_mode=bm,label_mode=mode,inputs=encode(values),calls=calls,result=result)
     }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-discrete-label-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete positional label callback records\n')
