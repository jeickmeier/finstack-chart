# FIX-GG04: exact Date/POSIXct inputs to non-color scale label functions.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
cases <- list()
for(channel in c('size','alpha','linewidth'))
 for(kind in c('date','datetime'))
 for(zone in if(kind=='date')'UTC' else c('UTC','America/New_York'))
  for(epoch in if(kind=='date')1704067200 else c(1710046800,1730606400))
   for(population in c('spaced','constant','missing','all_missing','empty'))
    for(control in c('automatic','explicit','empty','format'))
     for(mode in c('indexed','missing','short','empty')) {
      offsets <- switch(population,spaced=c(0,1,2,3),constant=c(1,1),missing=c(NA,0,3),all_missing=c(NA_real_,NA_real_),empty=numeric())
      convert <- function(v) if(kind=='date') as.Date(v+epoch/86400,origin='1970-01-01') else as.POSIXct(v*3600+epoch,origin='1970-01-01',tz=zone)
      values <- convert(offsets);calls <- list()
      label_function <- function(x) {
       calls[[length(calls)+1L]] <<- list(values=encode(as.numeric(x)),class=as.list(class(x)),zone=as.list(attr(x,'tzone')),names=as.list(names(x)))
       labels <- paste0(seq_along(x),'/',length(x))
       switch(mode,indexed=labels,missing=replace(labels,seq_along(x)%%2==0,NA_character_),short=head(labels,1),empty=character())
      }
      args <- list(labels=label_function)
      if(kind=='datetime')args$timezone<-zone
      if(control=='explicit')args$breaks<-convert(c(-1,0,1,3,4,NA))
      if(control=='empty')args$breaks<-convert(numeric())
      if(control=='format')args$date_labels<-if(kind=='date')'%Y-%m-%d'else'%H:%M'
      result <- tryCatch(suppressWarnings({
       scale <- do.call(get(paste0('scale_',channel,'_',kind)),args)
       mapping <- aes(x,1,group=1);mapping[[channel]] <- quote(v)
       geometry <- if(channel=='linewidth')geom_line()else geom_point()
       b <- ggplot_build(ggplot(data.frame(x=seq_along(values),v=values),mapping)+geometry+scale)
       keys <- lapply(b$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label)))
       list(keys=unname(keys),limits=encode(b$plot$scales$get_scales(channel)$get_limits()))
      }),error=function(e)list(error=conditionMessage(e)))
      cases[[length(cases)+1L]] <- list(channel=channel,kind=kind,zone=zone,epoch=epoch,population=population,control=control,label_mode=mode,inputs=encode(as.numeric(values)),calls=calls,result=result)
     }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/noncolor-temporal-label-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'non-color temporal label callback records\n')
