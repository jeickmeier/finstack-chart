# FIX-GG04: transformed NaN limits retain missing-endpoint population semantics.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
cases <- list()
encode <- function(v) lapply(v, function(x) if (is.na(x)) NULL else if (is.infinite(x)) if (x > 0) 'Infinity' else '-Infinity' else unname(x))
for (kind in c('continuous','identity','binned_nice','binned_equal'))
  for (tr in c('sqrt','log10'))
    for (population in c('finite','empty','missing','infinite'))
      for (limits in list(c(-1,10),c(-10,-1),c(10,-1),c(NA_real_,-1),c(-1,NA_real_)))
        for (explicit in c(FALSE,TRUE)) {
          result <- tryCatch(suppressWarnings({
            args <- list(transform=tr, limits=limits,
              breaks=if(explicit) c(-1,0,1,2,10,20) else waiver())
            s <- switch(kind,
              continuous=do.call(scale_colour_continuous,args),
              identity=do.call(scale_size_identity,c(args,list(guide='legend'))),
              binned_nice=do.call(scale_colour_steps,args),
              binned_equal=do.call(scale_colour_steps,c(args,list(nice.breaks=FALSE))))
            x <- switch(population,finite=c(1,10),empty=numeric(),missing=c(NA_real_,NA_real_),infinite=c(Inf,-Inf))
            s$train(s$transform(x))
            b <- s$get_breaks()
            list(breaks=encode(b), labels=as.list(unname(s$get_labels(b))),
              limits=encode(s$get_limits()), visible=as.list(is.finite(scales::oob_censor_any(b,s$get_limits()))))
          }), error=function(e) list(error=conditionMessage(e)))
          cases[[length(cases)+1]] <- list(kind=kind,transform=tr,population=population,
            limits=encode(limits),explicit=explicit,result=result)
        }
primary <- list()
for (kind in c('continuous','binned_nice')) for (tr in c('sqrt','log10'))
  for (population in c('empty','missing','infinite')) for(limits in list(c(-1,10),c(1,10))) {
    result <- tryCatch(suppressWarnings({
      x <- switch(population,empty=numeric(),missing=c(NA_real_,NA_real_),infinite=c(Inf,-Inf))
      s <- if(kind=='continuous') scale_colour_continuous(transform=tr,limits=limits) else scale_colour_steps(transform=tr,limits=limits)
      p <- ggplot(data.frame(x=seq_along(x),v=x),aes(x,1,colour=v))+geom_point()+s
      b <- ggplot_build(p)
      g <- ggplot_gtable(b)
      boxes <- g$grobs[grepl('guide-box',g$layout$name)]
      list(colors=as.list(unname(b$data[[1]]$colour)),guides=sum(!vapply(boxes,inherits,logical(1),'zeroGrob')))
    }),error=function(e)list(error=conditionMessage(e)))
    primary[[length(primary)+1]] <- list(kind=kind,transform=tr,population=population,limits=as.list(limits),result=result)
  }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases,primary=primary),
  'fixtures/parity/ggplot2/transformed-missing-limits.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'transformed missing-limit records\n')
