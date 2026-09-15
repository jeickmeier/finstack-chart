# FIX-GG04: temporal positional OOB vectors use transformed reference units.
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
for (family in c('duration', 'date', 'datetime'))
 for (kind in c('point_x', 'point_y', 'mean'))
  for (limit_mode in c('automatic', 'explicit'))
   for (population in c('ordinary', 'missing', 'empty'))
    for (mode in c('default', 'reverse', 'index', 'short', 'empty', 'null')) {
     origin <- switch(family, duration = 20000, date = 20000, datetime = 1700000000)
     inputs <- switch(population, ordinary = origin + c(-2, 1, 4, 10, 14),
       missing = origin + c(NA, 1, NA, 4, NA), empty = numeric())
     typed <- function(x) switch(family, duration = hms::as_hms(x),
       date = as.Date(x, origin = '1970-01-01'),
       datetime = as.POSIXct(x, origin = '1970-01-01', tz = 'UTC'))
     groups <- rep(c(1, 1, 2, 2, 3), length.out = length(inputs))
     axis <- if (kind == 'point_x') 'x' else 'y'
     calls <- list()
     oob <- function(x, range) {
      calls[[length(calls) + 1L]] <<- list(values = encode(as.numeric(x)),
        range = encode(as.numeric(range)), names = as.list(names(x)),
        value_class = as.list(class(x)), range_class = as.list(class(range)))
      switch(mode, default = scales::oob_censor(x, range), reverse = rev(x),
        index = as.numeric(seq_along(x)), short = head(x, 1), empty = numeric(), null = NULL)
     }
     result <- tryCatch(suppressWarnings({
      ctor <- get(paste0('scale_', axis, '_', switch(family,
        duration = 'time', date = 'date', datetime = 'datetime')))
      args <- list(oob = oob)
      if (limit_mode == 'explicit') args$limits <- typed(origin + c(1, 10))
      if (family == 'datetime') args$timezone <- 'UTC'
      mapping <- if (axis == 'x') aes(v, i) else aes(i, v)
      data <- data.frame(v = typed(inputs), i = if (kind == 'mean') groups else seq_along(inputs))
      layer <- if (kind == 'mean') stat_summary(fun = mean, geom = 'point') else geom_point()
      built <- ggplot_build(ggplot(data, mapping) + layer + do.call(ctor, args))
      panel <- built$layout$panel_params[[1]][[axis]]
      list(mapped = encode(built$data[[1]][[axis]]), limits = encode(panel$limits),
        breaks = encode(panel$breaks), labels = encode(panel$get_labels()))
     }), error = function(e) list(error = conditionMessage(e)))
     cases[[length(cases) + 1L]] <- list(family = family, kind = kind, axis = axis,
       origin = origin, population = population, limit_mode = limit_mode, mode = mode,
       inputs = encode(inputs), groups = encode(groups), calls = calls, result = result)
    }
stopifnot(length(cases) == 324L)
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases),
 'fixtures/parity/ggplot2/positional-pipeline-temporal.json',
 auto_unbox = TRUE, pretty = TRUE, digits = 15, null = 'null', na = 'null')
dev.off()
cat('captured', length(cases), 'temporal positional OOB cases\n')
