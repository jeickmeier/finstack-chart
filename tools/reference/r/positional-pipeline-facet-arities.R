# FIX-GG04: fixed/free facet populations and ordered positional vector stages.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
pdf(file = tempfile(fileext = '.pdf'))
encode <- function(x) lapply(unname(x), function(v) {
 if (is.na(v)) NULL else if (is.infinite(v)) {if (v > 0) 'Infinity' else '-Infinity'} else v
})
cases <- list()
for (kind in c('point'))
 for (scales in c('free_y'))
  for (transform in c('identity'))
   for (population in c('singleton','one_panel','two_singletons','two_pairs','uneven'))
    for (mode in c('default','reverse','index','short','empty','null')) {
     inputs <- switch(population, singleton=c(2), one_panel=c(2,4,6), two_singletons=c(2,4), two_pairs=c(2,4,6,8), uneven=c(2,4,6,8,10))
     facets <- if (population %in% c('singleton','one_panel')) rep('A',length(inputs)) else rep(c('A','B'),length.out=length(inputs))
     groups <- rep(c(1,1,2,2,3), length.out = length(inputs))
     calls <- list()
     oob <- function(x, range) {
      calls[[length(calls)+1L]] <<- list(values=encode(x),range=encode(range),names=as.list(names(x)))
      switch(mode,default=scales::oob_censor(x,range),reverse=rev(x),index=as.numeric(seq_along(x)),short=head(x,1),empty=numeric(),null=NULL)
     }
     result <- tryCatch(suppressWarnings({
      d <- data.frame(v=inputs,g=groups,i=seq_along(inputs),f=factor(facets,levels=c('A','B','C')))
      p <- ggplot(d,if (kind=='point') aes(i,v) else aes(g,v)) +
       (if (kind=='point') geom_point() else stat_summary(fun=mean,geom='point')) +
       scale_y_continuous(transform=transform,oob=oob) + facet_wrap(vars(f),scales=scales,drop=FALSE)
      b <- ggplot_build(p)
      panel <- lapply(seq_along(b$layout$panel_params),function(i) {
       p <- b$layout$panel_params[[i]]$y
       selected <- as.integer(b$data[[1]]$PANEL)==i
       list(panel=i,x=encode(b$data[[1]]$x[selected]),mapped=encode(b$data[[1]]$y[selected]),limits=encode(p$limits),breaks=encode(p$breaks),labels=encode(p$get_labels()))
      })
      list(panels=panel)
     }),error=function(e)list(error=conditionMessage(e)))
     cases[[length(cases)+1L]] <- list(kind=kind,scales=scales,transform=transform,population=population,mode=mode,inputs=encode(inputs),facets=as.list(facets),groups=encode(groups),calls=calls,result=result)
    }
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),'fixtures/parity/ggplot2/positional-pipeline-facet-arities.json',auto_unbox=TRUE,pretty=TRUE,digits=15,null='null',na='null')
dev.off()
cat('captured',length(cases),'positional facet cases\n')
