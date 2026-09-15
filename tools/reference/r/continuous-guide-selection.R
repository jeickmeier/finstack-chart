# GG-04 scale-side guide selection; GG-05 owns complete guide presentation.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x), function(v) {
  if (is.na(v)) NULL else if (is.infinite(v)) {
    if (v > 0) 'Infinity' else '-Infinity'
  } else v
})
pdf(file = tempfile('continuous-guides-', fileext = '.pdf'))
cases <- list()
for (channel in c('colour', 'size', 'alpha'))
for (population in c('ordinary', 'missing', 'empty'))
for (guide in c('default', 'none', 'legend', 'colourbar', 'bins', 'coloursteps'))
for (break_mode in if (guide == 'colourbar') c('default', 'null', 'empty', 'outside') else if (guide %in% c('bins', 'coloursteps')) c('default', 'null', 'empty', 'uneven') else c('default', 'null', 'empty')) {
  inputs <- switch(population, ordinary = c(0, 1, 2, 3, 4),
                   missing = c(NA, 1, Inf, 3, -Inf), empty = numeric())
  calls <- list()
  palette <- function(x) {
    calls[[length(calls) + 1L]] <<- encode(x)
    if (channel == 'colour') {
      ifelse(is.na(x), NA_character_, ifelse(x < .5, '#ff0000', '#0000ff'))
    } else if (channel == 'size') 1 + 4 * x else x
  }
  warnings <- character()
  selected <- NULL
  result <- tryCatch(withCallingHandlers({
    arguments <- list(aesthetics = channel, palette = palette, limits = c(0, 4))
    if (guide != 'default') arguments$guide <- guide
    if (break_mode != 'default') arguments['breaks'] <- list(
      if (break_mode == 'null') NULL else if (break_mode == 'outside') c(-2, 6) else if (break_mode == 'uneven') c(0, .5, 3, 4) else numeric())
    scale <- do.call(continuous_scale, arguments)
    selected <- scale$guide
    mapping <- aes(x, 1)
    mapping[[channel]] <- quote(v)
    built <- ggplot_build(ggplot(data.frame(x = seq_along(inputs), v = inputs), mapping) +
                            geom_point() + scale)
    raw_breaks <- built$plot$scales$get_scales(channel)$get_breaks()
    parsed <- if (guide %in% c('bins', 'coloursteps')) ggplot2:::parse_binned_breaks(built$plot$scales$get_scales(channel), raw_breaks) else NULL
    grob <- ggplotGrob(built)
    grid::grid.draw(grob)
    list(mapped = encode(built$data[[1]][[channel]]), raw_breaks = encode(raw_breaks),
         guides = unname(lapply(built$plot$guides$params, function(g) {
           list(values = encode(g$key$.value), labels = encode(g$key$.label),
                source_values = if (guide == 'bins') encode(sort(unique(c(parsed$limits, parsed$breaks)), na.last = NA)) else if (guide == 'coloursteps') encode(parsed$breaks[!is.na(parsed$breaks)]) else NULL,
                mapped = encode(g$key[[channel]]),
                decor_values = encode(g$decor$value), decor_colors = encode(g$decor$colour))
         })))
  }, warning = function(w) {
    warnings <<- c(warnings, conditionMessage(w))
    invokeRestart('muffleWarning')
  }), error = function(e) list(error = conditionMessage(e)))
  cases[[length(cases) + 1L]] <- list(channel = channel, population = population,
    guide = guide, breaks = break_mode, selected = selected, inputs = encode(inputs),
    calls = calls, warnings = as.list(warnings), result = result)
}
guide_contracts <- setNames(lapply(c('GuideLegend', 'GuideColourbar', 'GuideBins', 'GuideColoursteps'), function(name) {
  object <- getExportedValue('ggplot2', name)
  setNames(lapply(c('extract_key', 'extract_decor'), function(method) {
    value <- object[[method]]
    if (inherits(value, 'ggproto_method')) value <- environment(value)$f
    paste(deparse(value, width.cutoff = 500L), collapse = '\n')
  }), c('extract_key', 'extract_decor'))
}), c('GuideLegend', 'GuideColourbar', 'GuideBins', 'GuideColoursteps'))
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', guide_contracts = guide_contracts, parse_binned_breaks = paste(deparse(ggplot2:::parse_binned_breaks, width.cutoff = 500L), collapse = '\n'), cases = cases),
  'fixtures/parity/ggplot2/continuous-guide-selection.json', auto_unbox = TRUE,
  pretty = TRUE, digits = 17, null = 'null', na = 'null')
invisible(dev.off())
cat('captured', length(cases), 'continuous guide-selection draws\n')
