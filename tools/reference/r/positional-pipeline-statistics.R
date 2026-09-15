# FIX-GG04: positional vector callbacks straddle an actual reducing statistic.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
pdf(file = tempfile(fileext = '.pdf'))
encode <- function(x) lapply(unname(x), function(v) {
  if (is.na(v)) NULL else if (is.infinite(v)) {
    if (v > 0) 'Infinity' else '-Infinity'
  } else v
})
cases <- list()
for (transform in c('identity', 'reverse', 'log10'))
 for (population in c('ordinary', 'missing', 'empty'))
  for (mode in c('default', 'reverse', 'index', 'short', 'empty', 'null')) {
   inputs <- switch(population, ordinary = c(-2, 1, 4, 10, 14),
                    missing = c(NA, 1, Inf, 4, -Inf), empty = numeric())
   groups <- rep(c(1, 1, 2, 2, 3), length.out = length(inputs))
   calls <- list()
   oob <- function(x, range) {
    calls[[length(calls) + 1L]] <<- list(values = encode(x), range = encode(range), names = as.list(names(x)))
    switch(mode, default = scales::oob_censor(x, range), reverse = rev(x),
           index = as.numeric(seq_along(x)), short = head(x, 1), empty = numeric(), null = NULL)
   }
   result <- tryCatch(suppressWarnings({
    built <- ggplot_build(ggplot(data.frame(v = inputs, g = groups), aes(g, v)) +
      stat_summary(fun = mean, geom = 'point') +
      scale_y_continuous(limits = c(1, 10), transform = transform, oob = oob))
    panel <- built$layout$panel_params[[1]]$y
    list(x = encode(built$data[[1]]$x), mapped = encode(built$data[[1]]$y),
         limits = encode(panel$limits), breaks = encode(panel$breaks),
         labels = encode(panel$get_labels()))
   }), error = function(e) list(error = conditionMessage(e)))
   cases[[length(cases) + 1L]] <- list(transform = transform, population = population,
      mode = mode, inputs = encode(inputs), groups = encode(groups), calls = calls, result = result)
  }
jsonlite::write_json(list(reference = 'ggplot2 4.0.3', cases = cases),
 'fixtures/parity/ggplot2/positional-pipeline-statistics.json',
 auto_unbox = TRUE, pretty = TRUE, digits = 15, null = 'null', na = 'null')
dev.off()
cat('captured', length(cases), 'positional statistic OOB cases\n')
