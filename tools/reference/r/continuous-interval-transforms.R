# GG-04 interval guide keys across pointwise and vector-coupled transforms.
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
for (transform in c('log10', 'cardinality', 'center'))
for (channel in c('colour', 'size', 'alpha'))
for (population in c('ordinary', 'missing'))
for (palette_mode in if (population == 'missing') c('full', 'index') else 'full')
for (guide in c('bins', 'coloursteps'))
for (break_mode in c('default', 'uneven')) {
  inputs <- switch(population, ordinary = c(1, 2, 4, 8, 16),
                   missing = c(NA, 2, Inf, 8, -Inf), empty = numeric())
  calls <- list()
  palette <- function(x) {
    calls[[length(calls) + 1L]] <<- encode(x)
    if (palette_mode == 'index') x <- seq_along(x) / length(x)
    if (channel == 'colour') {
      ifelse(is.na(x), NA_character_, ifelse(x < .5, '#ff0000', '#0000ff'))
    } else if (channel == 'size') 1 + 4 * x else x
  }
  warnings <- character()
  selected <- NULL
  result <- tryCatch(withCallingHandlers({
    tr <- switch(transform,
      log10 = scales::transform_log10(),
      cardinality = scales::new_transform('cardinality', function(x) x + length(x), function(x) x - length(x)),
      center = scales::new_transform('center', function(x) x - mean(x, na.rm = TRUE), function(x) x))
    arguments <- list(aesthetics = channel, palette = palette, limits = c(1, 16), transform = tr)
    if (guide != 'default') arguments$guide <- guide
    if (break_mode != 'default') arguments['breaks'] <- list(
      if (break_mode == 'null') NULL else if (break_mode == 'outside') c(-2, 6) else if (break_mode == 'uneven') c(1, 2, 8, 16) else numeric())
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
    list(scale_limits = encode(built$plot$scales$get_scales(channel)$get_limits()), mapped = encode(built$data[[1]][[channel]]), raw_breaks = encode(raw_breaks),
         guides = unname(lapply(built$plot$guides$params, function(g) {
           list(values = encode(g$key$.value), labels = encode(g$key$.label),
                scale_values = if (guide == 'bins') encode(sort(unique(c(parsed$limits, parsed$breaks)), na.last = NA)) else if (guide == 'coloursteps') encode(parsed$breaks[!is.na(parsed$breaks)]) else NULL,
                mapped = encode(g$key[[channel]]),
                decor_values = encode(g$decor$value), decor_colors = encode(g$decor$colour))
         })))
  }, warning = function(w) {
    warnings <<- c(warnings, conditionMessage(w))
    invokeRestart('muffleWarning')
  }), error = function(e) list(error = conditionMessage(e)))
  cases[[length(cases) + 1L]] <- list(transform = transform, palette_mode = palette_mode, channel = channel, population = population,
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
  'fixtures/parity/ggplot2/continuous-interval-transforms.json', auto_unbox = TRUE,
  pretty = TRUE, digits = 17, null = 'null', na = 'null')
invisible(dev.off())
cat('captured', length(cases), 'continuous interval-transform draws\n')
