# FIX-GG04: positional OOB functions run at both population stages.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x), function(v) {
  if (is.na(v)) NULL else if (is.infinite(v)) {
    if (v > 0) 'Infinity' else '-Infinity'
  } else v
})
cases <- list()
for (axis in c('x', 'y'))
 for (transform in c('identity', 'reverse', 'log10'))
  for (population in c('ordinary', 'missing', 'empty'))
   for (mode in c('default', 'reverse', 'index', 'short', 'empty', 'null')) {
    inputs <- switch(population, ordinary = c(-2, 1, 4, 10, 14),
                     missing = c(NA, 1, Inf, 4, -Inf), empty = numeric())
    calls <- list()
    oob <- function(x, range) {
      calls[[length(calls) + 1L]] <<- list(values = encode(x),
        range = encode(range), names = as.list(names(x)))
      switch(mode, default = scales::oob_censor(x, range), reverse = rev(x),
             index = as.numeric(seq_along(x)), short = head(x, 1),
             empty = numeric(), null = NULL)
    }
    result <- tryCatch(suppressWarnings({
      scale <- do.call(if (axis == 'x') scale_x_continuous else scale_y_continuous,
                       list(limits = c(1, 10), transform = transform, oob = oob))
      mapping <- if (axis == 'x') aes(v, seq_along(v)) else aes(seq_along(v), v)
      built <- ggplot_build(ggplot(data.frame(v = inputs), mapping) + geom_point() + scale)
      panel <- built$layout$panel_params[[1]][[axis]]
      list(mapped = encode(built$data[[1]][[axis]]), limits = encode(panel$limits),
           breaks = encode(panel$breaks), labels = encode(panel$get_labels()))
    }), error = function(e) list(error = conditionMessage(e)))
    cases[[length(cases) + 1L]] <- list(axis = axis, transform = transform,
      population = population, mode = mode, inputs = encode(inputs), calls = calls,
      result = result)
   }
jsonlite::write_json(list(reference = 'ggplot2 4.0.3', cases = cases),
  'fixtures/parity/ggplot2/positional-pipeline-functions.json',
  auto_unbox = TRUE, pretty = TRUE, digits = 15, null = 'null', na = 'null')
cat('captured', length(cases), 'positional OOB cases\n')
