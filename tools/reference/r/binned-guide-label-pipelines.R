# FIX-GG04/GG-05 boundary: explicit guide class changes key evaluation before rendering.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0)'Infinity' else '-Infinity' else v)
pdf(file=tempfile("pipeline-reference-",fileext=".pdf"))
cases <- list()
for(family in 'binned')
 for(limit_mode in 'full')
 for(channel in c('colour','size','alpha'))
  for(population in c('ordinary','missing','empty'))
   for(mode in c('default','rescale_index'))
    for(guide in c('bins','coloursteps'))
     for(label_mode in c('indexed','short','missing')) {
     inputs <- switch(population,ordinary=c(-2,1,4,10,14),missing=c(NA,1,Inf,4,-Inf),empty=numeric())
     calls <- list()
     record <- function(operation,x,range=NULL) {
      calls[[length(calls)+1L]] <<- list(operation=operation,values=encode(x),range=encode(range),names=as.list(names(x)))
     }
     oob <- function(x,range) {
      record('oob',x,range)
      if(mode=='oob_reverse')return(rev(x))
      if(mode=='oob_index')return(as.numeric(seq_along(x)))
      if(family=='continuous')scales::oob_censor(x,range) else scales::oob_squish(x,range)
     }
     rescaler <- function(x,from) {
      record('rescaler',x,from)
      if(mode=='rescale_index')return(seq_along(x)/max(length(x),1))
      if(mode=='rescale_empty')return(numeric())
      if(mode=='rescale_null')return(NULL)
      result<-scales::rescale(x,from=from)
      if(mode=='rescale_reverse')rev(result) else if(mode=='rescale_short')head(result,1) else result
     }
     palette <- function(x) {
      record('palette',x)
      if(channel=='colour')ifelse(is.na(x),NA_character_,ifelse(x<.5,'#ff0000','#0000ff')) else if(channel=='size')1+4*x else x
     }
     labels <- function(x) {
      record('labels',x)
      result <- paste0(seq_along(x),'/',length(x))
      if(label_mode=='short')head(result,1) else if(label_mode=='missing') { result[seq_along(result)%%2==0L]<-NA_character_;result } else result
     }
     result <- tryCatch(suppressWarnings({
      scale<-do.call(if(family=='continuous')continuous_scale else binned_scale,list(aesthetics=channel,palette=palette,limits=switch(limit_mode,full=c(1,10),automatic=NULL,callback=function(x)x),oob=oob,rescaler=rescaler,guide=guide,labels=labels))
      mapping<-aes(x,1);mapping[[channel]]<-quote(v)
      built<-ggplot_build(ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point()+scale)
      grob<-ggplotGrob(built); grid::grid.draw(grob)
      panel<-grob$grobs[[which(grob$layout$name=='panel')]]
      points<-Filter(function(g)inherits(g,'points'),panel$children)
      point_count<-sum(vapply(points,function(g)length(g$x),integer(1)))
      point_colours<-unlist(lapply(points,function(g)g$gp$col),use.names=FALSE)
      list(point_count=point_count,point_colours=encode(point_colours),mapped=encode(built$data[[1]][[channel]]),keys=unname(lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label),mapped=encode(g$key[[channel]])))))
     }),error=function(e)list(error=conditionMessage(e)))
     cases[[length(cases)+1L]]<-list(family=family,channel=channel,population=population,mode=mode,guide_kind=guide,limit_mode=limit_mode,label_mode=label_mode,inputs=encode(inputs),calls=calls,result=result)
    }
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),'fixtures/parity/ggplot2/binned-guide-label-pipelines.json',auto_unbox=TRUE,pretty=TRUE,digits=15,null='null',na='null')
cat('captured',length(cases),'drawn constructor OOB/rescaler cases\n')

invisible(dev.off())
