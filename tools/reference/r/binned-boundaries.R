# FIX-GG04: bounded integer break counts and transformed missing-limit replacement.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
cases <- list()
encode <- function(v) lapply(v, function(x) if (is.na(x)) NULL else if (is.infinite(x)) if (x > 0) 'Infinity' else '-Infinity' else unname(x))
record <- function(tr, domain, limits, count, nice) {
  result <- tryCatch(suppressWarnings({
    s <- scale_colour_steps(transform=tr, limits=limits, n.breaks=count, nice.breaks=nice)
    s$train(s$transform(domain))
    b <- s$get_breaks()
    list(breaks=encode(b), labels=as.list(unname(s$get_labels(b))),
         limits=encode(s$get_limits()), visible=as.list(is.finite(scales::oob_censor_any(b, s$get_limits()))))
  }), error=function(e) list(error=conditionMessage(e)))
  cases[[length(cases)+1]] <<- list(transform=tr, domain=as.list(domain),
    limits=if(is.null(limits)) NULL else as.list(limits), count=count, nice=nice, result=result)
}
for (tr in c('identity','sqrt','log10','reverse'))
  for (domain in list(c(1,10),c(.8,8.4),c(4,4)))
    for (full in c(FALSE,TRUE))
      for (count in c(0,1,2,3,5))
        for (nice in c(FALSE,TRUE)) {
          # Zero nice counts can fail to terminate in labeling's reference search.
          # The portable API rejects these; never invoke that search in this oracle.
          if (count == 0 && nice) next
          record(tr, domain, if(full) domain else NULL, count, nice)
        }
for (tr in c('sqrt','log10'))
  for (limits in list(c(-1,10),c(-10,-1),c(10,-1)))
    for (nice in c(FALSE,TRUE)) record(tr,c(1,10),limits,5,nice)
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),
  'fixtures/parity/ggplot2/binned-boundaries.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'binned count and transformed-limit records\n')
