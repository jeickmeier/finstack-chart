# FIX-GG04: Date/POSIXct break-function inputs, count forwarding and typed results.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
default_names <- "--default-names" %in% commandArgs(trailingOnly=TRUE)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
metadata <- function(x) list(values=encode(as.numeric(x)),class=as.list(class(x)),zone=as.list(attr(x,'tzone')),names=as.list(names(x)))
cases <- list()
for(channel in c('colour','size'))
 for(kind in c('date','datetime'))
  for(zone in if(kind=='date')'UTC' else c('UTC','America/New_York'))
   for(epoch in if(kind=='date')1704067200 else c(1710046800,1730606400))
    for(population in c('spaced','constant','missing','all_missing','empty'))
     for(limits in c('none','full'))
      for(signature in if(default_names)'n'else c('limits','n','n.breaks'))
       for(count in if(default_names)'three'else c('none','three','zero'))
        for(mode in if(default_names)'mixed'else c('domain','mixed','empty','null','numeric')) {
         offsets <- switch(population,spaced=c(0,1,2,3),constant=c(1,1),missing=c(NA,0,3),all_missing=c(NA_real_,NA_real_),empty=numeric())
         typed <- function(v) if(kind=='date')as.Date(v,origin='1970-01-01')else as.POSIXct(v,origin='1970-01-01',tz=zone)
         convert <- function(v)typed(if(kind=='date')v+epoch/86400 else v*3600+epoch)
         values <- convert(offsets);calls <- list();label_calls <- list()
         evaluate <- function(x,provided=NULL,effective=NULL) {
          calls[[length(calls)+1L]] <<- c(metadata(x),list(count=provided,effective=effective))
          v<-as.numeric(x)
          switch(mode,domain=x,mixed=setNames(typed(c(v[2],mean(v),v[1],v[1],NA,Inf,-Inf)),c('last','middle','first','again','missing','positive','negative')),empty=typed(numeric()),null=NULL,numeric=v)
         }
         breaks <- switch(signature,limits=function(x)evaluate(x),n=function(x,n=7)evaluate(x,if(missing(n))NULL else n,n),n.breaks=function(x,n.breaks=9)evaluate(x,if(missing(n.breaks))NULL else n.breaks,n.breaks))
         labels <- function(x) {
          label_calls[[length(label_calls)+1L]] <<- metadata(x)
          paste0(seq_along(x),'/',length(x))
         }
         result <- tryCatch(suppressWarnings({
          args <- list(breaks=breaks,labels=if(default_names)waiver()else labels,limits=if(limits=='full')convert(c(0,3))else NULL,n.breaks=switch(count,none=NULL,three=3,zero=0))
          if(kind=='datetime')args$timezone <- zone
          if(channel=='size')args$range <- c(1,6)
          scale <- do.call(get(paste0('scale_',channel,'_',kind)),args)
          mapping <- aes(x,1);mapping[[channel]] <- quote(v)
          built <- ggplot_build(ggplot(data.frame(x=seq_along(values),v=values),mapping)+geom_point()+scale)
          keys <- lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label)))
          list(keys=unname(keys),mapped=encode(built$data[[1]][[channel]]))
         }),error=function(e)list(error=conditionMessage(e)))
         cases[[length(cases)+1L]] <- list(channel=channel,kind=kind,zone=zone,epoch=epoch,population=population,limits=limits,signature=signature,count=count,mode=mode,inputs=encode(as.numeric(values)),calls=calls,label_calls=label_calls,result=result)
        }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),if(default_names)'fixtures/parity/ggplot2/temporal-break-default-names.json'else'fixtures/parity/ggplot2/temporal-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'temporal break function reference builds\n')
