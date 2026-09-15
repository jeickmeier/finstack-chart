# FIX-GG04: typed Date/POSIXct minor callbacks, arity and width override precedence.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v))NULL else if(is.numeric(v)&&!is.finite(v))if(v>0)'Infinity'else '-Infinity'else v)
metadata <- function(x)list(values=encode(as.numeric(x)),class=as.list(class(x)),zone=as.list(attr(x,'tzone')),names=as.list(names(x)))
overrides <- '--overrides' %in% commandArgs(TRUE)
cases <- list()
for(kind in c('date','datetime'))for(zone in if(kind=='date')'UTC'else c('UTC','America/New_York'))for(epoch in if(kind=='date')1704067200 else c(1710046800,1730606400))for(population in c('spaced','constant','missing','all_missing','empty'))for(limits in c('none','full'))for(major in c('automatic','explicit','empty','null'))for(signature in c('one','two'))for(mode in c('domain','mixed','majors','empty','null','numeric'))for(expand in c('default','zero')) {
 if(overrides && !(population %in% c('spaced','constant') && limits=='none' && major=='automatic' && signature=='two' && expand=='default'))next
 offsets <- switch(population,spaced=c(0,1,2,3),constant=c(1,1),missing=c(NA,0,3),all_missing=c(NA_real_,NA_real_),empty=numeric())
 typed <- function(v)if(kind=='date')as.Date(v,origin='1970-01-01')else as.POSIXct(v,origin='1970-01-01',tz=zone)
 convert <- function(v)typed(if(kind=='date')v+epoch/86400 else v*3600+epoch)
 values <- convert(offsets);calls <- list()
 evaluate <- function(x,b=NULL) {
  calls[[length(calls)+1L]] <<- list(limits=metadata(x),major=if(is.null(b))NULL else metadata(b))
  v <- as.numeric(x)
  switch(mode,domain=x,mixed=typed(c(v[2],mean(v),v[1],v[1],NA,Inf,-Inf)),majors=b,empty=typed(numeric()),null=NULL,numeric=v)
 }
 minor <- if(signature=='one')function(x)evaluate(x)else function(x,b)evaluate(x,b)
 result <- tryCatch(suppressWarnings({
  args <- list(minor_breaks=minor,limits=if(limits=='full')convert(c(0,3))else NULL)
  if(major=='explicit')args$breaks <- convert(c(3,1,0,0,NA,Inf,-Inf))
  if(major=='empty')args$breaks <- typed(numeric())
  if(major=='null')args['breaks'] <- list(NULL)
  if(expand=='zero')args$expand <- expansion(0)
  if(kind=='datetime')args$timezone <- zone
  if(overrides)args$date_minor_breaks <- if(kind=='date')'1 day'else '1 hour'
  built <- ggplot_build(ggplot(data.frame(x=values,y=rep(1,length(values))),aes(x,y))+geom_point()+do.call(if(kind=='date')scale_x_date else scale_x_datetime,args))
  panel <- built$layout$panel_params[[1]]$x
  list(major=encode(panel$get_breaks()),labels=encode(panel$get_labels()),minor=encode(panel$get_breaks_minor()),range=encode(panel$continuous_range))
 }),error=function(e)list(error=conditionMessage(e)))
 case <- list(kind=kind,zone=zone,epoch=epoch,population=population,limits=limits,major=major,signature=signature,mode=mode,expand=expand,inputs=encode(as.numeric(values)),calls=calls,result=result)
 if(overrides)case$control <- 'width'
 cases[[length(cases)+1L]] <- case
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),if(overrides)'fixtures/parity/ggplot2/positional-temporal-minor-break-overrides.json'else 'fixtures/parity/ggplot2/positional-temporal-minor-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'temporal positional minor callback builds\n')
