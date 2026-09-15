# FIX-GG04: numeric non-color guide candidates and label callbacks.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x), function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
cases <- list()
for(channel in c('size','alpha','linewidth'))
 for(kind in c('continuous','binned'))
  for(tr in c('identity','sqrt','log10','reverse'))
   for(pop in c('ordinary','constant','missing','all_missing','empty'))
    for(limits in c('none','full'))
     for(bm in c('auto','explicit','empty'))
      for(mode in c('indexed','missing','short','empty',if(kind=='binned')'default')) {
       values <- switch(pop,ordinary=c(1,4,10),constant=c(4,4),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric())
       calls <- list()
       label_function <- function(x) {
        calls[[length(calls)+1L]] <<- list(values=encode(x),names=as.list(names(x)))
        labels <- paste0(seq_along(x),'/',length(x))
        switch(mode,indexed=labels,missing=replace(labels,seq_along(x)%%2==0,NA_character_),short=head(labels,1),empty=character())
       }
       result <- tryCatch(suppressWarnings({
        constructor <- get(paste0('scale_',channel,'_',kind))
        scale <- constructor(transform=tr,limits=if(limits=='full')c(1,10)else NULL,
         breaks=if(bm=='explicit')c(-1,0,1,1,3,20,Inf,NA)else if(bm=='empty')numeric()else waiver(),labels=if(mode=='default')waiver()else label_function)
        data <- data.frame(x=seq_along(values),y=seq_along(values),v=values)
        mapping <- aes(x,y,group=1);mapping[[channel]] <- quote(v)
        geometry <- if(channel=='linewidth')geom_line()else geom_point()
        built <- ggplot_build(ggplot(data,mapping)+geometry+scale)
        keys <- lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label)))
        result <- list(keys=unname(keys))
        if(kind=='binned') {
         scale <- built$plot$scales$get_scales(channel)
         parsed <- ggplot2:::parse_binned_breaks(scale)
         result$boundaries <- if(is.null(parsed))list()else encode(sort(unique(c(parsed$limits,parsed$breaks)),na.last=NA))
        }
        result
       }),error=function(e)list(error=conditionMessage(e)))
       cases[[length(cases)+1L]] <- list(channel=channel,kind=kind,transform=tr,population=pop,limits=limits,break_mode=bm,label_mode=mode,inputs=encode(values),calls=calls,result=result)
      }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/noncolor-numeric-label-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'non-color numeric label callback records\n')
