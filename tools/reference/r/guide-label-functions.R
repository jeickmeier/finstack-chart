# FIX-GG04: label callbacks receive selected source-space vectors before censoring.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
column <- function(x) lapply(seq_along(x), function(i) {
  v <- x[[i]]
  if (is.na(v)) return(NULL)
  if (is.numeric(v) && !is.finite(v)) return(if (v > 0) 'Infinity' else '-Infinity')
  unname(v)
})
cases <- list()
capture <- function(scale, descriptor, mode) {
  calls <- list()
  scale$labels <- function(x) {
    calls[[length(calls) + 1L]] <<- list(values = column(x), names = column(names(x)))
    labels <- paste0(seq_along(x), '/', length(x))
    switch(mode, indexed = labels, missing = replace(labels, seq_along(x) %% 2 == 0, NA_character_),
           short = head(labels, 1), empty = character(),
           named = setNames(labels, rev(as.character(x))))
  }
  result <- tryCatch({
    b <- scale$get_breaks()
    l <- scale$get_labels(b)
    list(breaks = column(b), labels = column(l), label_names = column(names(l)))
  }, error = function(e) list(error = conditionMessage(e)))
  direct_calls <- calls
  calls <- list()
  values <- if (descriptor$kind == 'continuous') c(1, 4, 10) else
    switch(descriptor$population, ordinary = c('a', 'b', 'c'),
           nullable = c('a', 'b', NA), empty = character())
  data <- data.frame(x = seq_along(values), y = seq_along(values), v = values)
  build <- tryCatch({
    b <- ggplot_build(ggplot(data, aes(x, y, colour = v)) + geom_point() + scale$clone())
    keys <- lapply(b$plot$guides$params, function(g)
      list(values = column(g$key$.value), labels = column(g$key$.label)))
    list(keys = unname(keys))
  }, error = function(e) list(error = conditionMessage(e)))
  c(descriptor, list(label_mode = mode, calls = direct_calls, result = result,
                     build_calls = calls, build = build))
}
for (transform in c('identity', 'sqrt', 'log10', 'reverse'))
  for (limits in list(c(1, 10), c(4, 4)))
    for (break_mode in c('auto', 'explicit', 'empty'))
      for (mode in c('indexed', 'missing', 'short', 'empty', 'named')) {
        breaks <- switch(break_mode, auto = waiver(),
                         explicit = c(-1, 0, .1, 1, 5, 10, 20, Inf, NA, NaN), empty = numeric())
        scale <- scale_colour_gradient(limits = limits, transform = transform, breaks = breaks)
        cases[[length(cases) + 1L]] <- capture(scale,
          list(kind = 'continuous', transform = transform, limits = as.list(limits),
               break_mode = break_mode), mode)
      }
for (population in c('ordinary', 'nullable', 'empty'))
  for (break_mode in c('auto', 'explicit', 'named', 'empty'))
    for (mode in c('indexed', 'missing', 'short', 'empty', 'named')) {
      breaks <- switch(break_mode, auto = waiver(), explicit = c('z', 'b', 'b', NA, 'a'),
                       named = c(Z = 'z', B = 'b', B2 = 'b', M = NA, A = 'a'), empty = character())
      scale <- scale_colour_discrete(breaks = breaks)
      values <- switch(population, ordinary = c('a', 'b', 'c'), nullable = c('a', 'b', NA), empty = character())
      scale$train(values)
      cases[[length(cases) + 1L]] <- capture(scale,
        list(kind = 'discrete', population = population, break_mode = break_mode), mode)
    }
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases),
  'fixtures/parity/ggplot2/guide-label-functions.json', auto_unbox = TRUE,
  pretty = TRUE, digits = NA, null = 'null')
cat('PASS', length(cases), 'guide label callback records\n')
