# GG-04: explicit custom transform/inverse and default guide callback contracts.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3',
          as.character(packageVersion('scales')) == '1.4.0')
library(ggplot2)
encode <- function(v) lapply(unname(v), function(x) {
  if(is.nan(x)) return(list(number = 'NaN'))
  if(is.na(x)) return(list(number = 'NA'))
  if(is.infinite(x)) return(list(number = if(x > 0) 'Infinity' else '-Infinity'))
  x
})
make_transform <- function(family, custom) {
  if(family == 'unif') scales::transform_probability('unif', min = if(custom) -2 else 0, max = if(custom) 3 else 1) else
    scales::transform_probability('exp', rate = if(custom) 2 else .5)
}
capture_query <- function(value) tryCatch(value, error = function(e) list(error = conditionMessage(e)))
configurations <- list(); cases <- list()
pdf(file = tempfile('probability-transforms-', fileext = '.pdf'))
for(family in c('unif', 'exp')) for(custom in c(FALSE, TRUE)) {
  transform <- make_transform(family, custom)
  raw <- c(-Inf, -2, -.01, 0, .01, .25, .5, .75, .99, 1, 1.01, 3, Inf, NA_real_, NaN)
  configurations[[length(configurations) + 1L]] <- list(family = family, custom = custom, inputs = encode(raw),
    quantiles = encode(suppressWarnings(transform$transform(raw))),
    probabilities = encode(suppressWarnings(transform$inverse(raw))),
    forward = paste(deparse(transform$transform), collapse = '\n'),
    inverse = paste(deparse(transform$inverse), collapse = '\n'),
    breaks = paste(deparse(transform$breaks), collapse = '\n'),
    format = paste(deparse(transform$format), collapse = '\n'))
  for(route in c('position', 'paint', 'binned_paint')) for(population in c('ordinary', 'nonfinite', 'empty')) {
    x <- switch(population, ordinary = c(.01, .25, .5, .75, .99),
                nonfinite = c(.01, .25, .5, .75, .99, NA_real_, -Inf, Inf), empty = numeric())
    result <- tryCatch(suppressWarnings({
      p <- if(route == 'position') ggplot(data.frame(x), aes(x, 1)) + geom_point() +
        scale_x_continuous(transform = transform) else
        ggplot(data.frame(x, i = seq_along(x)), aes(i, 1, colour = x)) + geom_point() +
        if(route == 'paint') scale_colour_gradient(transform = transform) else scale_colour_steps(transform = transform)
      b <- ggplot_build(p); grid::grid.draw(ggplotGrob(p))
      panel <- b$layout$panel_params[[1]]$x
      scale <- if(route == 'position') b$layout$panel_scales_x[[1]] else b$plot$scales$get_scales('colour')
      # Direct scale/panel queries can reject after a successful empty chart draw.
      # Preserve their diagnostics independently of build/draw acceptance.
      list(build_draw = 'ok', mapped = if(route == 'position') encode(b$data[[1]]$x) else as.list(b$data[[1]]$colour),
           range = encode(panel$continuous_range), positions = capture_query(encode(panel$break_positions())),
           panel_labels = capture_query(as.list(panel$get_labels())), breaks = capture_query(encode(scale$get_breaks())),
           labels = capture_query(as.list(scale$get_labels())), limits = capture_query(encode(scale$get_limits())))
    }), error = function(e) list(error = conditionMessage(e)))
    cases[[length(cases) + 1L]] <- list(configuration = length(configurations) - 1L,
      route = route, population = population, inputs = encode(x), result = result)
  }
}
invisible(dev.off())
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',
  constructor = paste(deparse(scales::transform_probability), collapse = '\n'),
  configurations = configurations, cases = cases),
  'fixtures/parity/ggplot2/probability-transforms.json', auto_unbox = TRUE,
  pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('Captured', length(cases), 'custom-transform chart cases; reference only.\n')
