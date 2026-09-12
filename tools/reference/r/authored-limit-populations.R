# FIX-GG04: exceptional authored limits and partial-limit orientation.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(v,function(x) if(is.nan(x)) 'NaN' else if(is.na(x)) NULL else if(is.infinite(x)) if(x>0) 'Infinity' else '-Infinity' else unname(x))
cases <- list()
inputs <- c(-Inf,-1,0,1,4,10,Inf,NA_real_)
limits_list <- list(c(1,Inf),c(-Inf,10),c(-Inf,Inf),c(Inf,Inf),c(-Inf,-Inf),
  c(NaN,10),c(1,NaN),c(NaN,NaN),c(NA_real_,10),c(1,NA_real_),
  c(10,NA_real_),c(NA_real_,1),c(10,1),c(NA_real_,Inf),c(Inf,NA_real_),
  c(NA_real_,-Inf),c(-Inf,NA_real_),c(Inf,10),c(1,-Inf),c(NA_real_,NA_real_),NULL)
for(kind in c('continuous','identity','binned_nice','binned_equal'))
  for(tr in c('identity','sqrt','log10','reverse'))
    for(population in c('finite','empty','missing','infinite'))
      for(limits in limits_list)
        for(mode in c('auto','explicit')) {
          result <- tryCatch(suppressWarnings({
            args <- list(transform=tr,limits=limits,breaks=if(mode=='auto')waiver()else c(-Inf,-1,0,1,4,10,Inf))
            s <- switch(kind,
              continuous=do.call(scale_colour_gradient,args),
              identity=do.call(scale_size_identity,c(args,list(guide='legend'))),
              binned_nice=do.call(scale_colour_steps,args),
              binned_equal=do.call(scale_colour_steps,c(args,list(nice.breaks=FALSE))))
            x <- switch(population,finite=c(1,10),empty=numeric(),missing=c(NA_real_,NA_real_),infinite=c(Inf,-Inf))
            s$train(s$transform(x))
            guide <- tryCatch({
              b <- s$get_breaks()
              list(breaks=encode(b),labels=as.list(unname(s$get_labels(b))),
                visible=as.list(is.finite(scales::oob_censor_any(b,s$get_limits()))))
            },error=function(e)list(error=conditionMessage(e)))
            mapping <- tryCatch({
              mapped <- s$map(s$transform(inputs))
              list(values=if(kind=='identity')encode(mapped)else as.list(unname(mapped)))
            },error=function(e)list(error=conditionMessage(e)))
            list(guide=guide,mapping=mapping)
          }),error=function(e)list(guide=list(error=conditionMessage(e)),mapping=list(error=conditionMessage(e))))
          cases[[length(cases)+1]] <- c(list(kind=kind,transform=tr,population=population,
            limits=if(is.null(limits))NULL else encode(limits),mode=mode,cuts=if(mode=='explicit')encode(c(-Inf,-1,0,1,4,10,Inf))else NULL,
            inputs=encode(inputs)),result)
        }
primary <- list()
for (case in cases) {
  if (case$kind == 'identity') next
  decode <- function(v) vapply(v, function(x) if(is.null(x)) NA_real_ else if(is.character(x)) switch(x,Infinity=Inf,`-Infinity`=-Inf,`NaN`=NaN) else x, numeric(1))
  result <- tryCatch(suppressWarnings({
    args <- list(transform=case$transform, limits=if(is.null(case$limits))NULL else decode(case$limits),
      breaks=if(case$mode=='auto')waiver()else decode(case$cuts))
    s <- switch(case$kind,continuous=do.call(scale_colour_gradient,args),
      binned_nice=do.call(scale_colour_steps,args),
      binned_equal=do.call(scale_colour_steps,c(args,list(nice.breaks=FALSE))))
    v <- switch(case$population,finite=c(1,10),empty=numeric(),missing=c(NA_real_,NA_real_),infinite=c(Inf,-Inf))
    p <- ggplot(data.frame(x=seq_along(v),v=v),aes(x,1,colour=v))+geom_point()+s
    # Separate scale mapping from colorbar construction, which GG-05 owns.
    b <- ggplot_build(p + guides(colour='none'))
    guide <- tryCatch({
      g <- ggplot_gtable(ggplot_build(p))
      boxes <- g$grobs[grepl('guide-box',g$layout$name)]
      list(count=sum(!vapply(boxes,inherits,logical(1),'zeroGrob')))
    },error=function(e)list(error=conditionMessage(e)))
    list(colors=as.list(unname(b$data[[1]]$colour)),guide=guide)
  }),error=function(e)list(error=conditionMessage(e)))
  primary[[length(primary)+1]] <- c(case[c('kind','transform','population','limits','mode','cuts')],list(result=result))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases,primary=primary),
  'fixtures/parity/ggplot2/authored-limit-populations.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'authored limit population records\n')
