# FIX-GG04: temporal interval callbacks and explicit date-format precedence.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x), function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
pdf(file=tempfile('temporal-guides-',fileext='.pdf'))
cases <- list()
for(kind in c('date','datetime'))
 for(channel in c('colour','size','alpha'))
  for(limits in c('none','full'))
  for(population in c('ordinary','constant','empty'))
   for(guide in c('bins','coloursteps'))
    for(break_mode in c('auto','explicit','empty'))
     for(label_mode in c('indexed','missing','short','empty','named'))
      for(format_mode in c('none','explicit')) {
     values <- switch(population,ordinary=c(0,2,8,10),constant=c(4,4),empty=numeric())
     to_time <- function(x) if(kind=='date')as.Date(x,origin='2020-01-01')else as.POSIXct(x*3600,origin='2020-01-01',tz='UTC')
     calls <- list(); label_calls <- list(); warnings <- character(); selected <- NULL
     label_function <- function(x) {
      label_calls[[length(label_calls)+1L]] <<- list(values=encode(as.numeric(x)),class=as.list(class(x)),zone=as.list(attr(x,'tzone')),names=as.list(names(x)))
      labels <- paste0(seq_along(x),'/',length(x))
      switch(label_mode,indexed=labels,missing=replace(labels,seq_along(x)%%2==0,NA_character_),short=head(labels,1),empty=character(),named=setNames(labels,rev(as.character(x))))
     }
     result <- tryCatch(withCallingHandlers({
      arguments <- if(limits=='full')list(limits=to_time(c(0,10)))else list()
      arguments$guide <- guide
      arguments$labels <- label_function
      if(format_mode=='explicit')arguments$date_labels <- if(kind=='date')'%d/%m'else'%Hh%M'
      if(break_mode!='auto')arguments['breaks'] <- list(if(break_mode=='explicit')to_time(c(-1,0,1,1,3,20,NA))else to_time(numeric()))
      scale <- do.call(get(paste0('scale_',channel,'_',kind)),arguments)
      selected <- scale$guide
      original_palette <- scale$palette
      if(is.function(original_palette))scale$palette <- function(x) {calls[[length(calls)+1L]] <<- encode(x); original_palette(x)}
      mapping <- aes(x,1);mapping[[channel]] <- quote(v)
      built <- ggplot_build(ggplot(data.frame(x=seq_along(values),v=to_time(values)),mapping)+geom_point()+scale)
      resolved <- built$plot$scales$get_scales(channel)
      raw_breaks <- resolved$get_breaks()
      parsed <- if(guide %in% c('bins','coloursteps'))ggplot2:::parse_binned_breaks(resolved,raw_breaks)else NULL
      grob <- ggplotGrob(built);grid::grid.draw(grob)
      list(limits=encode(resolved$get_limits()),mapped=encode(built$data[[1]][[channel]]),raw_breaks=encode(raw_breaks),guides=unname(lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label),source_values=if(guide=='bins')encode(sort(unique(c(parsed$limits,parsed$breaks)),na.last=NA))else if(guide=='coloursteps')encode(parsed$breaks[!is.na(parsed$breaks)])else NULL,mapped=encode(g$key[[channel]]),decor_values=encode(g$decor$value),decor_colors=encode(g$decor$colour)))))
     },warning=function(w){warnings <<- c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
     cases[[length(cases)+1L]] <- list(kind=kind,channel=channel,limits=limits,population=population,guide=guide,breaks=break_mode,selected=selected,inputs=encode(values),label_mode=label_mode,format_mode=format_mode,label_calls=label_calls,calls=calls,warnings=as.list(warnings),result=result)
    }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/temporal-interval-label-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
invisible(dev.off())
cat('captured',length(cases),'temporal interval label draws\n')
