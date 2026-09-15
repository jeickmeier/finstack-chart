# FIX-GG04: public scale constructors with vector/count palette functions.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v)if(is.na(v))NULL else if(is.numeric(v)&&!is.finite(v))if(v>0)'Infinity'else'-Infinity'else v)
cases <- list()
for(family in c('continuous','discrete','binned'))
 for(channel in c('colour','size','alpha'))
  for(pop in c('ordinary','constant','missing','all_missing','empty'))
   for(limits in c('none','full'))
    for(mode in c('full','short','empty','null','named','missing')) {
     values <- if(family=='discrete')switch(pop,ordinary=c('a','b','c'),constant=c('b','b'),missing=c(NA,'a','c'),all_missing=c(NA_character_,NA_character_),empty=character())else switch(pop,ordinary=c(1,4,10),constant=c(4,4),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric())
     calls <- list()
     palette <- function(x) {
      calls[[length(calls)+1L]] <<- list(values=encode(x),names=as.list(names(x)))
      n <- if(family=='discrete')x else length(x)
      t <- if(family=='discrete')seq_len(n)/max(n,1)else x
      result <- if(channel=='colour')ifelse(is.na(t),NA_character_,ifelse(t<.5,'#ff0000','#0000ff'))else if(channel=='size')1+4*t else t
      if(mode=='short')result <- result[1]
      if(mode=='empty')result <- result[0]
      if(mode=='null')result <- NULL
      if(mode=='named')names(result) <- rep(c('c','b','a','d'),length.out=length(result))
      if(mode=='missing'&&length(result)>1)result[2] <- NA
      result
     }
     result <- tryCatch(suppressWarnings({
      args <- list(aesthetics=channel,palette=palette,limits=if(limits=='full')if(family=='discrete')c('a','b','c','d')else c(1,10)else NULL,guide='legend')
      scale <- do.call(switch(family,continuous=continuous_scale,discrete=discrete_scale,binned=binned_scale),args)
      mapping <- aes(x,1);mapping[[channel]] <- quote(v)
      built <- ggplot_build(ggplot(data.frame(x=seq_along(values),v=values),mapping)+geom_point()+scale)
      keys <- lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label),mapped=encode(g$key[[channel]])))
      list(mapped=encode(built$data[[1]][[channel]]),keys=unname(keys))
     }),error=function(e)list(error=conditionMessage(e)))
     cases[[length(cases)+1L]] <- list(family=family,channel=channel,population=pop,limits=limits,mode=mode,inputs=encode(values),calls=calls,result=result)
    }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/palette-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'palette function reference builds\n')
