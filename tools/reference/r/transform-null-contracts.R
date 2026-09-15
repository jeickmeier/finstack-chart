# GG-04: dependency-backed transform controls accepted by as.transform().
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3',
          as.character(packageVersion('scales')) == '1.4.0')
library(scales)
encode <- function(v) lapply(unname(v), function(x) {
  if(is.nan(x)) return(list(number = 'NaN'))
  if(is.na(x)) return(list(number = 'NA'))
  if(is.infinite(x)) return(list(number = if(x > 0) 'Infinity' else '-Infinity'))
  x
})
exports <- sort(grep('^transform_', getNamespaceExports('scales'), value = TRUE))
contracts <- lapply(exports, function(name) {
  f <- getExportedValue('scales', name)
  list(name = name, formals = lapply(formals(f), function(x) paste(deparse(x), collapse = '\n')),
       body = paste(deparse(body(f), width.cutoff = 500L), collapse = '\n'))
})
specs <- list(
  list('asinh', list()), list('asn', list()), list('atanh', list()),
  list('boxcox', list(p = 0)), list('boxcox', list(p = .5)), list('boxcox', list(p = -1, offset = 2)),
  list('exp', list()), list('exp', list(base = 2)), list('identity', list()),
  list('log', list()), list('log', list(base = .5)), list('log10', list()), list('log2', list()),
  list('log1p', list()), list('logit', list()), list('probit', list()),
  list('modulus', list(p = 0)), list('modulus', list(p = .5)), list('modulus', list(p = 2, offset = 2)),
  list('pseudo_log', list()), list('pseudo_log', list(sigma = .5, base = 10)),
  list('reciprocal', list()), list('reverse', list()), list('sqrt', list()),
  list('yj', list(p = 0)), list('yj', list(p = 1)), list('yj', list(p = 2)),
  list('probability', list(distribution = 'norm', mean = 2, sd = 3)),
  list('probability', list(distribution = 'logis', location = 2, scale = 3)))
evaluate <- function(f, x) tryCatch({y <- suppressWarnings(f(x));list(values=encode(y),is_null=is.null(y))},error=function(e)list(error=conditionMessage(e)))
cases <- lapply(specs,function(spec){
 t <- do.call(getExportedValue('scales',paste0('transform_',spec[[1]])),spec[[2]])
 list(constructor=spec[[1]],args=spec[[2]],forward_null=evaluate(t$transform,NULL),inverse_null=evaluate(t$inverse,NULL),forward_empty=evaluate(t$transform,numeric()),inverse_empty=evaluate(t$inverse,numeric()))
})
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/transform-null-contracts.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat(length(cases),'NULL transform configurations captured\n')
