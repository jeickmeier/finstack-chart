# FIX-GG04: guide metadata and mapping failures are distinct on degenerate domains.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
pdf(tempfile(fileext='.pdf'))
encode <- function(v) lapply(v,function(x) if(is.nan(x)) 'NaN' else if(is.na(x)) NULL else if(is.infinite(x)) if(x>0) 'Infinity' else '-Infinity' else unname(x))
cases <- list()
for(kind in c('continuous','identity','binned_nice','binned_equal'))
  for(tr in c('identity','sqrt','log10','reverse'))
    for(limits in list(c(0,10),c(0,0),c(4,4),c(1,10)))
      for(mode in c('auto','explicit','single','empty')) {
        args <- list(transform=tr,limits=limits,breaks=switch(mode,
          auto=waiver(),explicit=c(-1,0,1,2,4,4,10,20),single=c(4,4),empty=numeric()))
        s <- suppressWarnings(switch(kind,
          continuous=do.call(scale_colour_gradient,args),
          identity=do.call(scale_size_identity,c(args,list(guide='legend'))),
          binned_nice=do.call(scale_colour_steps,args),
          binned_equal=do.call(scale_colour_steps,c(args,list(nice.breaks=FALSE)))))
        s$train(suppressWarnings(s$transform(c(1,10))))
        guide <- tryCatch(suppressWarnings({
          b <- s$get_breaks()
          list(breaks=encode(b),labels=as.list(unname(s$get_labels(b))),
            visible=as.list(is.finite(scales::oob_censor_any(b,s$get_limits()))))
        }),error=function(e)list(error=conditionMessage(e)))
        mapping <- tryCatch(suppressWarnings({
          mapped <- s$map(s$transform(c(-1,0,1,4,10,NA_real_)))
          list(values=if(kind=='identity') encode(mapped) else as.list(unname(mapped)))
        }),error=function(e)list(error=conditionMessage(e)))
        draw <- if(kind=='identity') NULL else tryCatch(suppressWarnings({
          fresh <- switch(kind,
            continuous=do.call(scale_colour_gradient,args),
            binned_nice=do.call(scale_colour_steps,args),
            binned_equal=do.call(scale_colour_steps,c(args,list(nice.breaks=FALSE))))
          p <- ggplot(data.frame(x=1:6,v=c(-1,0,1,4,10,NA_real_)),aes(x,1,colour=v)) +
            geom_point() + fresh
          # The existing continuous proof authors the typed key guide, not a colourbar.
          if(kind=='continuous') p <- p + guides(colour=guide_legend())
          ggplotGrob(p)
          list(ok=TRUE)
        }),error=function(e)list(error=conditionMessage(e)))
        cases[[length(cases)+1]] <- list(kind=kind,transform=tr,limits=as.list(limits),mode=mode,guide=guide,mapping=mapping,draw=draw)
      }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),
  'fixtures/parity/ggplot2/degenerate-bin-mapping.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'degenerate mapping and guide records\n')
