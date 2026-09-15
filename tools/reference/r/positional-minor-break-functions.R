# FIX-GG04: positional numeric minor callbacks, arity and transformed inputs.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity'else '-Infinity'else v)
joint <- '--joint' %in% commandArgs(TRUE)
cases <- list()
for(trans in c('identity','sqrt','log10','reverse'))
 for(population in c('spaced','constant','missing','all_missing','empty'))
  for(limits in c('none','full'))
   for(major in if(joint)c('function_fixed','function_domain')else c('automatic','explicit','empty','null'))
    for(signature in c('one','two'))
     for(mode in c('domain','mixed','majors','empty','null'))
      for(expand in c('default','zero')) {
       values <- switch(population,spaced=c(1,2,5,10),constant=c(5,5),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric())
       calls <- list();major_calls <- list()
       evaluate <- function(x,b=NULL) {
        calls[[length(calls)+1L]] <<- list(limits=encode(x),major=if(is.null(b))NULL else encode(b))
        switch(mode,domain=x,mixed=c(x[2],mean(x),x[1],x[1],NA,Inf,-Inf),majors=b,empty=numeric(),null=NULL)
       }
       minor <- if(signature=='one')function(x)evaluate(x)else function(x,b)evaluate(x,b)
       result <- tryCatch(suppressWarnings({
        args <- list(transform=trans,limits=if(limits=='full')c(1,10)else NULL,minor_breaks=minor)
        if(joint)args$breaks <- function(x) { major_calls[[length(major_calls)+1L]] <<- encode(x);if(major=='function_fixed')c(10,5,1,1,NA,Inf,-Inf)else x }
        if(major=='explicit')args$breaks <- c(10,5,1,1,NA,Inf,-Inf)
        if(major=='empty')args$breaks <- numeric()
        if(major=='null')args['breaks'] <- list(NULL)
        if(expand=='zero')args$expand <- expansion(0)
        built <- ggplot_build(ggplot(data.frame(x=values,y=rep(1,length(values))),aes(x,y))+geom_point()+do.call(scale_x_continuous,args))
        panel <- built$layout$panel_params[[1]]$x
        list(major=encode(panel$get_breaks()),labels=encode(panel$get_labels()),minor=encode(panel$get_breaks_minor()),range=encode(panel$continuous_range))
       }),error=function(e)list(error=conditionMessage(e)))
       case <- list(transform=trans,population=population,limits=limits,major=major,signature=signature,mode=mode,expand=expand,inputs=encode(values),calls=calls,result=result)
       if(joint)case$major_calls <- major_calls
       cases[[length(cases)+1L]] <- case
      }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),if(joint)'fixtures/parity/ggplot2/positional-minor-break-joint-functions.json'else 'fixtures/parity/ggplot2/positional-minor-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'numeric positional minor callback builds\n')
