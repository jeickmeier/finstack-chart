# GG-04 generic discrete/binned NULL and empty-break guide selection.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x), function(v) {
  if (is.na(v)) NULL else if (is.infinite(v)) {
    if (v > 0) 'Infinity' else '-Infinity'
  } else v
})
pdf(file = tempfile('generic-guides-', fileext = '.pdf'))
cases <- list()
for (family in c('discrete', 'binned'))
for (channel in c('colour', 'size', 'alpha'))
for (population in c('ordinary', 'missing', 'empty'))
for (limit_mode in c('automatic', 'fixed'))
for (guide in c('default', 'none', 'legend'))
for (break_mode in c('default', 'null', 'empty')) {
  inputs <- if (family == 'discrete') {
    switch(population, ordinary = c('a', 'b', 'c'), missing = c(NA, 'b', 'c'),
           empty = character())
  } else switch(population, ordinary = c(0, 1, 2, 3, 4),
                missing = c(NA, 1, Inf, 3, -Inf), empty = numeric())
  calls <- list()
  palette <- function(x) {
    calls[[length(calls) + 1L]] <<- encode(x)
    if (family == 'discrete') x <- if (x > 0) seq(0, 1, length.out = x) else numeric()
    if (channel == 'colour') {
      ifelse(is.na(x), NA_character_, ifelse(x < .5, '#ff0000', '#0000ff'))
    } else if (channel == 'size') 1 + 4 * x else x
  }
  warnings <- character()
  selected <- NULL
  result <- tryCatch(withCallingHandlers({
    arguments <- list(aesthetics = channel, palette = palette)
    if (limit_mode == 'fixed') arguments$limits <- if (family == 'discrete') c('a', 'b', 'c') else c(0, 4)
    if (guide != 'default') arguments$guide <- guide
    if (break_mode != 'default') arguments['breaks'] <- list(
      if (break_mode == 'null') NULL else if (family == 'discrete') character() else numeric())
    scale <- do.call(if (family == 'discrete') discrete_scale else binned_scale, arguments)
    selected <- scale$guide
    mapping <- aes(x, 1)
    mapping[[channel]] <- quote(v)
    built <- ggplot_build(ggplot(data.frame(x = seq_along(inputs), v = inputs), mapping) +
                            geom_point() + scale)
    trained <- built$plot$scales$get_scales(channel)
    grob <- ggplotGrob(built)
    grid::grid.draw(grob)
    list(mapped = encode(built$data[[1]][[channel]]), limits = encode(trained$get_limits()),
         raw_breaks = encode(trained$get_breaks()),
         guides = unname(lapply(built$plot$guides$params, function(g) {
           list(values = encode(g$key$.value), labels = encode(g$key$.label),
                mapped = encode(g$key[[channel]]))
         })))
  }, warning = function(w) {
    warnings <<- c(warnings, conditionMessage(w))
    invokeRestart('muffleWarning')
  }), error = function(e) list(error = conditionMessage(e)))
  cases[[length(cases) + 1L]] <- list(family = family, channel = channel,
    population = population, limit_mode = limit_mode, guide = guide,
    breaks = break_mode, selected = selected, inputs = encode(inputs),
    calls = calls, warnings = as.list(warnings), result = result)
}
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases),
  'fixtures/parity/ggplot2/generic-guide-suppression.json', auto_unbox = TRUE,
  pretty = TRUE, digits = 17, null = 'null', na = 'null')
invisible(dev.off())
cat('captured', length(cases), 'generic guide-suppression draws\n')
