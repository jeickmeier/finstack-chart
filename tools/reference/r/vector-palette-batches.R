# FIX-GG04: whole-vector callbacks must preserve per-layer order and uniqueness.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x), function(v) if(is.na(v)) NULL else v)
cases <- list()
for (family in c('continuous', 'binned'))
 for (channel in c('colour', 'size', 'alpha'))
  for (population in c('repeated', 'missing', 'empty_first'))
   for (mode in c('index', 'length', 'first'))
    for (guide in c('hidden', 'automatic', 'restricted')) {
     inputs <- switch(population,
       repeated=list(c(10,1,10,4), c(4,10,1,4)),
       missing=list(c(NA,10,1,NA,4), c(4,NA,1,10)),
       empty_first=list(numeric(), c(10,4,1,10)))
     calls <- list()
     palette <- function(x) {
       calls[[length(calls)+1L]] <<- list(values=encode(x), names=as.list(names(x)))
       t <- switch(mode,
         index=seq_along(x)/max(length(x),1),
         length=rep(length(x)/10,length(x)),
         first=rep(if(length(x))x[1]else NA_real_,length(x)))
       if (channel=='colour') {
         ifelse(is.na(t),NA_character_,ifelse(t<.5,'#ff0000','#0000ff'))
       } else if (channel=='size') 1+4*t else t
     }
     result <- tryCatch(suppressWarnings({
       scale <- do.call(switch(family,continuous=continuous_scale,binned=binned_scale),
         list(aesthetics=channel,palette=palette,limits=c(1,10),
              guide=if(guide=='hidden')'none'else'legend',
              breaks=if(guide=='restricted')c(10,1,4)else waiver()))
       mapping <- aes(x,1); mapping[[channel]] <- quote(v)
       plot <- ggplot()+scale
       for (values in inputs) {
         plot <- plot+geom_point(data=data.frame(x=seq_along(values),v=values),mapping=mapping)
       }
       built <- ggplot_build(plot)
       keys <- lapply(built$plot$guides$params,function(g)
         list(values=encode(g$key$.value),labels=encode(g$key$.label),mapped=encode(g$key[[channel]])))
       list(mapped=lapply(built$data,function(d)encode(d[[channel]])),keys=unname(keys))
     }), error=function(e)list(error=conditionMessage(e)))
     cases[[length(cases)+1L]] <- list(family=family,channel=channel,
       population=population,mode=mode,guide_mode=guide,
       inputs=lapply(inputs,encode),calls=calls,result=result)
    }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),
 'fixtures/parity/ggplot2/vector-palette-batches.json',
 auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'vector-sensitive palette builds\n')
