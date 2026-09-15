# FIX-GG04: discrete and binned positional minor callback semantics.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v))NULL else if(is.numeric(v)&&!is.finite(v))if(v>0)'Infinity'else '-Infinity'else v)
cases <- list()
for(family in c('discrete','binned'))for(population in c('spaced','constant','missing','all_missing','empty'))for(major in c('automatic','empty','null'))for(signature in c('one','two'))for(mode in c('domain','mixed','majors','empty','null'))for(expand in c('default','zero')) {
 values <- if(family=='discrete')switch(population,spaced=c('a','b','c'),constant=c('b','b'),missing=c(NA,'a','c'),all_missing=c(NA_character_,NA_character_),empty=character())else switch(population,spaced=c(1,2,5,10),constant=c(5,5),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric())
 calls <- list()
 evaluate <- function(x,b=NULL) {
  calls[[length(calls)+1L]] <<- list(limits=encode(x),major=if(is.null(b))NULL else encode(b))
  switch(mode,domain=x,mixed=c(x[2],mean(x),x[1],x[1],-100,100,NA,Inf,-Inf),majors=b,empty=numeric(),null=NULL)
 }
 minor <- if(signature=='one')function(x)evaluate(x)else function(x,b)evaluate(x,b)
 result <- tryCatch(suppressWarnings({
  args <- list(minor_breaks=minor)
  if(major=='empty')args$breaks <- if(family=='discrete')character()else numeric()
  if(major=='null')args['breaks'] <- list(NULL)
  if(expand=='zero')args$expand <- expansion(0)
  built <- ggplot_build(ggplot(data.frame(x=values,y=rep(1,length(values))),aes(x,y))+geom_point()+do.call(if(family=='discrete')scale_x_discrete else scale_x_binned,args))
  panel <- built$layout$panel_params[[1]]$x
  list(major=encode(panel$get_breaks()),labels=encode(panel$get_labels()),minor=encode(panel$get_breaks_minor()),range=encode(panel$continuous_range))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]] <- list(family=family,population=population,major=major,signature=signature,mode=mode,expand=expand,inputs=encode(values),calls=calls,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-other-minor-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete and binned positional minor callback builds\n')
