# FIX-GG04: interval guides share continuous break selection and vector label calls.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
pdf(file=tempfile(fileext=".pdf"))
encode <- function(x) lapply(unname(x), function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
cases <- list()
for(channel in c('colour','size','alpha'))
 for(guide in c('bins','coloursteps'))
  for(tr in c('identity','log10'))
   for(pop in c('ordinary','constant','empty'))
    for(limits in c('none','full'))
     for(bm in c('auto','explicit','empty'))
      for(mode in c('indexed','missing','short','empty','named')) {
       values <- switch(pop,ordinary=c(1,4,10),constant=c(4,4),empty=numeric())
       calls <- list()
       label_function <- function(x) {
        calls[[length(calls)+1L]] <<- list(values=encode(x),names=as.list(names(x)))
        labels <- paste0(seq_along(x),'/',length(x))
        switch(mode,indexed=labels,missing=replace(labels,seq_along(x)%%2==0,NA_character_),short=head(labels,1),empty=character(),named=setNames(labels,rev(as.character(x))))
       }
       result <- tryCatch(suppressWarnings({
        constructor <- get(paste0('scale_',channel,'_continuous'))
        scale <- constructor(transform=tr,guide=guide,limits=if(limits=='full')c(1,10)else NULL,
         breaks=if(bm=='explicit')c(-1,0,1,1,3,20,Inf,NA)else if(bm=='empty')numeric()else waiver(),labels=label_function)
        mapping <- aes(x,1);mapping[[channel]] <- quote(v)
        built <- ggplot_build(ggplot(data.frame(x=seq_along(values),v=values),mapping)+geom_point()+scale)
        keys <- lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label)))
        scale <- built$plot$scales$get_scales(channel)
        parsed <- ggplot2:::parse_binned_breaks(scale)
        list(keys=unname(keys),boundaries=if(length(keys)==0||is.null(parsed))list()else if(guide=='bins')encode(sort(unique(c(parsed$limits,parsed$breaks)),na.last=NA))else encode(parsed$breaks[!is.na(parsed$breaks)]))
       }),error=function(e)list(error=conditionMessage(e)))
       record <- list(channel=channel,kind='interval',guide=guide,transform=tr,population=pop,limits=limits,break_mode=bm,label_mode=mode,inputs=encode(values),calls=calls,result=result)
       if(is.null(result$error) && mode=='indexed' && ((pop %in% c('ordinary','empty') && limits=='full' && bm=='auto') || (pop=='constant' && limits=='none' && bm=='empty'))) {
        record$draw <- tryCatch(suppressWarnings({ggplot_gtable(built);list(ok=TRUE)}),error=function(e)list(error=conditionMessage(e)))
       }
       cases[[length(cases)+1L]] <- record
      }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/continuous-interval-label-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('captured',length(cases),'continuous interval label callback records\n')
