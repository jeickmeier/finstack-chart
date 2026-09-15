# FIX-GG04: numeric break functions, optional counts, names, and label inputs.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
default_names <- "--default-names" %in% commandArgs(trailingOnly=TRUE)
encode <- function(x) lapply(unname(x),function(v)if(is.na(v))NULL else if(is.numeric(v)&&!is.finite(v))if(v>0)'Infinity'else'-Infinity'else v)
cases <- list()
for(channel in c('colour','size'))
 for(tr in c('identity','sqrt','log10','reverse'))
  for(pop in c('ordinary','constant','missing','all_missing','empty'))
   for(limits in c('none','full'))
    for(signature in if(default_names)'n'else c('limits','n','n.breaks'))
     for(count in if(default_names)'three'else c('none','three','zero'))
      for(mode in if(default_names)'mixed'else c('domain','mixed','empty','null')) {
       values <- switch(pop,ordinary=c(1,4,10),constant=c(4,4),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric())
       calls <- list();label_calls <- list()
       evaluate <- function(x,provided=NULL,effective=NULL) {
        calls[[length(calls)+1L]] <<- list(limits=encode(x),count=provided,effective=effective,names=as.list(names(x)))
        switch(mode,domain=x,mixed=setNames(c(x[2],mean(x),x[1],x[1],NA,Inf,-Inf),c('last','middle','first','again','missing','positive','negative')),empty=numeric(),null=NULL)
       }
       breaks <- switch(signature,limits=function(x)evaluate(x),n=function(x,n=7)evaluate(x,if(missing(n))NULL else n,n),n.breaks=function(x,n.breaks=9)evaluate(x,if(missing(n.breaks))NULL else n.breaks,n.breaks))
       labels <- function(x) {
        label_calls[[length(label_calls)+1L]] <<- list(values=encode(x),names=as.list(names(x)))
        paste0(seq_along(x),'/',length(x))
       }
       result <- tryCatch(suppressWarnings({
        args <- list(transform=tr,limits=if(limits=='full')c(1,10)else NULL,n.breaks=switch(count,none=NULL,three=3,zero=0),breaks=breaks,labels=if(default_names)waiver()else labels)
        scale <- if(channel=='size')do.call(continuous_scale,c(list(aesthetics='size',palette=scales::pal_area(c(1,6))),args))else do.call(scale_colour_continuous,args)
        mapping <- aes(x,1);mapping[[channel]] <- quote(v)
        built <- ggplot_build(ggplot(data.frame(x=seq_along(values),v=values),mapping)+geom_point()+scale)
        keys <- lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label)))
        list(keys=unname(keys))
       }),error=function(e)list(error=conditionMessage(e)))
       cases[[length(cases)+1L]] <- list(channel=channel,constructor=if(channel=='size')'continuous_scale'else'scale_colour_continuous',transform=tr,population=pop,limits=limits,signature=signature,count=count,mode=mode,inputs=encode(values),calls=calls,label_calls=label_calls,result=result)
      }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),if(default_names)'fixtures/parity/ggplot2/numeric-break-default-names.json'else'fixtures/parity/ggplot2/numeric-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'numeric break function reference builds\n')
