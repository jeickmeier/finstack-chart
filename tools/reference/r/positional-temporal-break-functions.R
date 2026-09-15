# FIX-GG04: Date/POSIXct primary break selection and default/registered labels.
# Positional constructors have no n.breaks formal; explicit counts set the inherited scale field.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)

encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
metadata <- function(x) list(values=encode(as.numeric(x)),class=as.list(class(x)),zone=as.list(attr(x,'tzone')),names=as.list(names(x)))
overrides <- '--overrides' %in% commandArgs(TRUE)
zero <- '--zero-range' %in% commandArgs(TRUE)
cases <- list()
for(channel in 'x')
 for(kind in c('date','datetime'))
  for(zone in if(kind=='date')'UTC' else c('UTC','America/New_York'))
   for(epoch in if(kind=='date')1704067200 else c(1710046800,1730606400))
    for(population in c('spaced','constant','missing','all_missing','empty'))
     for(limits in c('none','full'))
      for(signature in c('limits','n','n.breaks'))
       for(count in c('none','three','zero'))
        for(mode in c('domain','mixed','empty','null','numeric')) for(label_mode in c('indexed','automatic')) for(control in if(zero)'zero'else if(overrides)c('width','format','both')else 'none') {
         if(zero && !(population=='constant' && limits=='none' && signature=='n' && count=='three')) next
         if(overrides && !(population %in% c('spaced','constant') && limits=='none' && signature=='n' && count=='three' && mode %in% c('mixed','numeric','null'))) next
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
          args <- list(breaks=breaks,labels=if(label_mode=='automatic')waiver()else labels,limits=if(limits=='full')convert(c(0,3))else NULL)
          if(zero)args$expand <- expansion(0)
          if(control %in% c('width','both'))args$date_breaks <- if(kind=='date')'1 day'else '1 hour'
          if(control %in% c('format','both'))args$date_labels <- if(kind=='date')'%Y-%m-%d'else '%H:%M'
          if(kind=='datetime')args$timezone <- zone
          scale <- do.call(get(paste0('scale_',channel,'_',kind)),args)
          if(count!='none')scale$n.breaks <- switch(count,three=3,zero=0)
          mapping <- aes(v,y)
          built <- ggplot_build(ggplot(data.frame(y=rep(1,length(values)),v=values),mapping)+geom_point()+scale)
          panel<-built$layout$panel_params[[1]]$x
          list(breaks=encode(panel$get_breaks()),labels=encode(panel$get_labels()),range=encode(panel$continuous_range),mapped=encode(built$data[[1]]$x))
         }),error=function(e)list(error=conditionMessage(e)))
         case <- list(channel=channel,label_mode=label_mode,count_route=if(count=='none')'constructor default'else'inherited scale field',kind=kind,zone=zone,epoch=epoch,population=population,limits=limits,signature=signature,count=count,mode=mode,inputs=encode(as.numeric(values)),calls=calls,label_calls=label_calls,result=result)
         if(overrides || zero)case$control <- control
         cases[[length(cases)+1L]] <- case
        }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),if(zero)'fixtures/parity/ggplot2/positional-temporal-break-zero.json'else if(overrides)'fixtures/parity/ggplot2/positional-temporal-break-overrides.json'else 'fixtures/parity/ggplot2/positional-temporal-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'temporal positional break function reference builds\n')
