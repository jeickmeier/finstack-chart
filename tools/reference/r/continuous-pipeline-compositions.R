# FIX-GG04: public constructor OOB/rescaler vector contracts and evaluation order.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0)'Infinity' else '-Infinity' else v)
cases <- list()
for(family in 'continuous')
 for(transform in c('identity','reverse','log10'))
  for(limit_mode in c('automatic','full','reversed','constant','singleton','empty','null','missing'))
 for(channel in c('colour','size','alpha'))
  for(population in c('ordinary','missing','empty'))
   for(mode in c('default','oob_index','rescale_index','both_index'))
    for(guide in c('hidden','automatic')) {
     inputs <- switch(population,ordinary=c(1,4,10),missing=c(NA,1,Inf,4,-Inf),empty=numeric())
     calls <- list()
     record <- function(operation,x,range=NULL) {
      calls[[length(calls)+1L]] <<- list(operation=operation,values=encode(x),range=encode(range),names=as.list(names(x)))
     }
     oob <- function(x,range) {
      record('oob',x,range)
      if(mode=='oob_reverse')return(rev(x))
      if(mode %in% c('oob_index','both_index'))return(as.numeric(seq_along(x)))
      if(family=='continuous')scales::oob_censor(x,range) else scales::oob_squish(x,range)
     }
     rescaler <- function(x,from) {
      record('rescaler',x,from)
      if(mode %in% c('rescale_index','both_index'))return(seq_along(x)/max(length(x),1))
      if(mode=='rescale_empty')return(numeric())
      if(mode=='rescale_null')return(NULL)
      result<-scales::rescale(x,from=from)
      if(mode=='rescale_reverse')rev(result) else if(mode=='rescale_short')head(result,1) else result
     }
     palette <- function(x) {
      record('palette',x)
      if(channel=='colour')ifelse(is.na(x),NA_character_,ifelse(x<.5,'#ff0000','#0000ff')) else if(channel=='size')1+4*x else x
     }
     limit <- switch(limit_mode, automatic=NULL, full=c(1,10), reversed=c(10,1), constant=c(4,4), singleton=function(x)4, empty=function(x)numeric(), null=function(x)NULL, missing=function(x)c(NA,10))
     result <- tryCatch(suppressWarnings({
      scale<-do.call(if(family=='continuous')continuous_scale else binned_scale,list(aesthetics=channel,palette=palette,limits=limit,transform=transform,oob=oob,rescaler=rescaler,guide=if(guide=='hidden')'none'else'legend'))
      mapping<-aes(x,1);mapping[[channel]]<-quote(v)
      built<-ggplot_build(ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point()+scale)
      list(mapped=encode(built$data[[1]][[channel]]),keys=unname(lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label),mapped=encode(g$key[[channel]])))))
     }),error=function(e)list(error=conditionMessage(e)))
     cases[[length(cases)+1L]]<-list(family=family,transform=transform,limit_mode=limit_mode,channel=channel,population=population,mode=mode,guide_mode=guide,inputs=encode(inputs),calls=calls,result=result)
    }
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),'fixtures/parity/ggplot2/continuous-pipeline-compositions.json',auto_unbox=TRUE,pretty=TRUE,digits=15,null='null',na='null')
cat('captured',length(cases),'transformed OOB/rescaler/limit compositions\n')
