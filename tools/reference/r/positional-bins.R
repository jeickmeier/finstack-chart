# FIX-GG04: positional bins before statistics and interval interpolation afterward.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(v,function(x) if(is.nan(x)) 'NaN' else if(is.na(x)) NULL else if(is.infinite(x)) if(x>0) 'Infinity' else '-Infinity' else unname(x))
cases <- list()
inputs <- c(-Inf,-1,0,1,2,4,5,8,10,11,Inf,NA_real_)
after <- c(-1,0,.5,1,1.5,2,2.5,3,4,5,NA_real_)
for(tr in c('identity','sqrt','log10','reverse'))
 for(mode in c('nice','equal','explicit','empty','none'))
  for(right in c(TRUE,FALSE))
   for(population in c('finite','constant','empty','missing','infinite'))
    for(limits in list(NULL,c(1,10),c(NA_real_,10),c(1,NA_real_),c(10,1),c(1,Inf)))
     for(show in c(FALSE,TRUE)) {
      args <- list(transform=tr,limits=limits,n.breaks=3,nice.breaks=mode!='equal',
        breaks=if(mode=='explicit')c(-1,1,2,4,10,20)else if(mode=='empty')numeric()else if(mode=='none')NULL else waiver(),
        right=right,show.limits=show)
      x <- switch(population,finite=c(1,10),constant=c(4,4),empty=numeric(),missing=c(NA_real_,NA_real_),infinite=c(Inf,-Inf))
      result <- tryCatch(suppressWarnings({
        s <- do.call(scale_x_binned,args);s$train(s$transform(x))
        breaks <- s$get_breaks();labels <- s$get_labels(breaks);actual_limits <- s$get_limits()
        before <- tryCatch(list(values=encode(s$map(s$transform(inputs)))),error=function(e)list(error=conditionMessage(e)))
        s$after.stat <- TRUE
        mapped <- tryCatch(list(values=encode(s$map(after))),error=function(e)list(error=conditionMessage(e)))
        list(breaks=encode(breaks),labels=as.list(unname(labels)),limits=encode(actual_limits),before=before,after=mapped)
      }),error=function(e)list(error=conditionMessage(e)))
      cases[[length(cases)+1]] <- list(transform=tr,mode=mode,right=right,population=population,
        limits=if(is.null(args$limits))NULL else encode(args$limits),show_limits=show,result=result)
     }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',inputs=encode(inputs),after=encode(after),cases=cases),
 'fixtures/parity/ggplot2/positional-bins.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'positional pre/post-stat bin records\n')
