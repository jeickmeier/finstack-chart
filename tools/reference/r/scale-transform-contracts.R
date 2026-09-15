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
inputs <- c(-Inf, -2, -1, -.5, 0, .001, .25, .5, .75, .999, 1, 2, 10, Inf, NA_real_, NaN)
cases <- lapply(specs, function(spec) {
  trans <- do.call(getExportedValue('scales', paste0('transform_', spec[[1]])), spec[[2]])
  evaluate <- function(f) tryCatch(list(values = encode(suppressWarnings(f(inputs)))), error = function(e) list(error = conditionMessage(e)))
  list(constructor = spec[[1]], args = spec[[2]], domain = encode(as.numeric(trans$domain)),
       inputs = encode(inputs), forward = evaluate(trans$transform), inverse = evaluate(trans$inverse))
})
library(ggplot2)
pdf(file = tempfile('scale-transforms-', fileext = '.pdf'))
plots <- list()
for(i in seq_along(specs)) for(population in c('ordinary', 'nonfinite')) for(route in c('position', 'paint')) {
  spec <- specs[[i]]
  x <- if(spec[[1]] %in% c('asn','logit','probit','probability')) c(.01,.25,.5,.75,.99) else
       if(spec[[1]] == 'atanh') c(-.9,-.5,0,.5,.9) else
       if(spec[[1]] %in% c('boxcox','log','log10','log2','sqrt','reciprocal')) c(.1,.5,1,2,4) else c(-.5,0,.5,1,2)
  if(population == 'nonfinite') x <- c(x, NA_real_, -Inf, Inf)
  result <- tryCatch(suppressWarnings({
    trans <- do.call(getExportedValue('scales', paste0('transform_', spec[[1]])), spec[[2]])
    p <- if(route == 'position') ggplot(data.frame(x), aes(x, 1)) + geom_point() + scale_x_continuous(transform = trans) else
         ggplot(data.frame(x, i = seq_along(x)), aes(i, 1, colour = x)) + geom_point() + scale_colour_gradient(transform = trans, guide = 'none')
    b <- ggplot_build(p); g <- ggplotGrob(b); grid::grid.draw(g)
    panel <- b$layout$panel_params[[1]]$x
    list(mapped = if(route == 'position') encode(b$data[[1]]$x) else as.list(b$data[[1]]$colour),
      range = encode(panel$continuous_range), breaks = encode(panel$get_breaks()),
      positions = encode(panel$break_positions()), labels = as.list(panel$get_labels()))
  }), error = function(e) list(error = conditionMessage(e)))
  plots[[length(plots) + 1L]] <- list(configuration = i - 1L, route = route, population = population,
    inputs = encode(x), result = result)
}
invisible(dev.off())
jsonlite::write_json(list(reference = paste('ggplot2 4.0.3 / R 4.6.1 / scales', packageVersion('scales')),
  contracts = contracts, cases = cases, plots = plots), 'fixtures/parity/ggplot2/scale-transform-contracts.json',
  auto_unbox = TRUE, pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('Captured', length(contracts), 'source contracts,', length(cases), 'numeric configurations and', length(plots), 'actual plots; no engine parity claim\n')
