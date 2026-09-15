# FIX-GG04: transform-owned minor breaks, transformed callback units and precedence.
suppressPackageStartupMessages(library(ggplot2))
stopifnot(as.character(packageVersion('ggplot2')) == '4.0.3', as.character(packageVersion('scales')) == '1.4.0')
encode <- function(x) lapply(x, function(v) {
  if(is.na(v)) return(list(number = if(is.nan(v)) 'NaN' else 'NA'))
  if(is.infinite(v)) return(list(number = if(v > 0) 'Infinity' else '-Infinity'))
  v
})
capture <- function(value) tryCatch(value, error = function(e) list(error = conditionMessage(e)))
configurations <- list(); cases <- list()
pdf(tempfile('registered-transform-minors-', fileext = '.pdf'))
for(family in c('affine', 'cubic')) for(mode in c('default', 'fixed', 'limits')) for(composed in c(FALSE, TRUE)) {
  trace <- list()
  minor <- function(b, limits, n) {
    trace[[length(trace) + 1L]] <<- list(major = encode(b), limits = encode(limits), subdivisions = n)
    if(mode == 'fixed') c(-1, 0, 1) else c(limits[1], mean(limits), limits[2])
  }
  forward <- if(family == 'affine') function(x) 2 * x + 3 else function(x) x^3
  inverse <- if(family == 'affine') function(x) (x - 3) / 2 else function(x) sign(x) * abs(x)^(1/3)
  args <- list(name = paste(family, mode), transform = forward, inverse = inverse)
  if(mode != 'default') args$minor_breaks <- minor
  trans <- do.call(scales::new_transform, args)
  if(composed) trans <- scales::transform_compose(trans, scales::transform_reverse())
  configurations[[length(configurations) + 1L]] <- list(family = family, mode = mode, composed = composed)
  for(population in c('ordinary', 'nonfinite', 'empty')) for(count in c(NA, 3, 7)) {
    overrides <- if(population == 'ordinary' && is.na(count) && !composed) c('default', 'explicit', 'hidden') else 'default'
    for(override in overrides) {
      trace <- list()
      x <- switch(population, ordinary = c(-2, -1, 0, 1, 2), nonfinite = c(-2, -1, 0, 1, 2, NA_real_, -Inf, Inf), empty = numeric())
      result <- capture(suppressWarnings({
        scale_args <- list(transform = trans, n.breaks = if(is.na(count)) NULL else count)
        if(override == 'explicit') scale_args$minor_breaks <- c(-1.5, 0.5)
        if(override == 'hidden') scale_args['minor_breaks'] <- list(NULL)
        p <- ggplot(data.frame(x), aes(x, 1)) + geom_point() + do.call(scale_x_continuous, scale_args)
        b <- ggplot_build(p); build_calls <- trace
        grid::grid.draw(ggplotGrob(p))
        panel <- b$layout$panel_params[[1]]$x
        list(build_draw = 'ok', range = encode(panel$continuous_range),
             major = encode(panel$get_breaks()), minor = encode(panel$minor_breaks),
             minor_positions = capture(encode(panel$break_positions_minor())),
             build_calls = build_calls)
      }))
      cases[[length(cases) + 1L]] <- list(configuration = length(configurations) - 1L, population = population,
        count = if(is.na(count)) NULL else count, override = override, inputs = encode(x), result = result)
    }
  }
}
invisible(dev.off())
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',
  source = paste(deparse(environment(ggplot2::ScaleContinuous$get_breaks_minor)$f), collapse = '\n'),
  configurations = configurations, cases = cases), 'fixtures/parity/ggplot2/registered-transform-minors.json',
  auto_unbox = TRUE, pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('Captured', length(cases), 'transform-owned minor-break charts; reference only.\n')
