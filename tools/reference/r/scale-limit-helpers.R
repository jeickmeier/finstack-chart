# GG2-03/FIX-GG04: helper dispatch and train-only expansion layer contracts.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
pdf(file = tempfile(fileext = '.pdf'))
encode <- function(x) lapply(unname(x), function(v) {
 if (is.na(v)) NULL else if (is.infinite(v)) {
  if (v > 0) 'Infinity' else '-Infinity'
 } else v
})
contracts <- lapply(c('limits', 'limits.numeric', 'limits.character', 'limits.factor',
 'limits.Date', 'limits.POSIXct', 'make_scale'), function(n) {
 f <- get(n, asNamespace('ggplot2'))
 list(name = n, formals = lapply(formals(f), function(x) paste(deparse(x), collapse = '\n')),
 body = paste(deparse(body(f), width.cutoff = 500L), collapse = '\n'))
})
cases <- list()
for (axis in c('x', 'y')) for (family in c('numeric', 'character', 'date', 'datetime'))
 for (mode in c('ascending', 'descending', 'lower_missing', 'both_missing', 'single', 'empty')) {
 vals <- switch(mode, ascending = c(1, 4), descending = c(4, 1), lower_missing = c(NA, 4),
  both_missing = c(NA, NA), single = 2, empty = numeric())
 convert <- switch(family, numeric = as.numeric, character = as.character,
  date = function(x) as.Date(x, origin = '1970-01-01'),
  datetime = function(x) as.POSIXct(x * 86400, origin = '1970-01-01', tz = 'UTC'))
 inputs <- convert(c(0, 1, 2, 4, 5)); limits <- convert(vals)
 result <- tryCatch(suppressWarnings({
  helper <- do.call(get(paste0(axis, 'lim')), list(limits))
  data <- data.frame(v = inputs, i = seq_along(inputs))
  mapping <- if (axis == 'x') aes(v, i) else aes(i, v)
  b <- ggplot_build(ggplot(data, mapping) + geom_point() + helper)
  p <- b$layout$panel_params[[1]][[axis]]
  list(scale_class = as.list(class(helper)), transform = helper$get_transformation()$name,
   mapped = encode(as.numeric(b$data[[1]][[axis]])), limits = encode(p$limits),
   breaks = encode(p$breaks), labels = encode(p$get_labels()),
   point_positions = encode(p$rescale(b$data[[1]][[axis]])),
   break_positions = encode(p$break_positions()))
 }), error = function(e) list(error = conditionMessage(e)))
 cases[[length(cases) + 1L]] <- list(axis = axis, family = family, mode = mode,
  inputs = encode(as.numeric(c(0, 1, 2, 4, 5))), authored = encode(vals), result = result)
}
expansions <- list()
for (family in c('numeric', 'category')) for (axes in c('x', 'y', 'xy', 'colour', 'colour_shared'))
 for (facet in c('none', 'fixed', 'free')) {
 d <- data.frame(x = c(1, 2, 3, 4), y = c(2, 4, 6, 8), g = c('A', 'A', 'B', 'B'))
 if (family == 'category') d$x <- d$y <- c('b', 'c', 'b', 'c')
 args <- if (axes == 'xy') list(x = if (family == 'numeric') c(-2, 10) else c('a', 'd'),
  y = if (family == 'numeric') 0 else 'a') else setNames(list(
   if (family == 'numeric') c(-2, 10) else c('a', 'd')), if (axes == 'colour_shared') 'colour' else axes)
 result <- tryCatch(suppressWarnings({
  helper <- do.call(expand_limits, args)
  p <- ggplot(d, if (axes == 'colour_shared') aes(x, y, colour = y) else aes(x, y)) + geom_point() + helper
  if (facet != 'none') p <- p + facet_wrap(~g, scales = facet)
  b <- ggplot_build(p)
  colour <- b$plot$scales$get_scales('colour')
  drawn <- ggplot_gtable(b)
  boxes <- which(grepl('^guide-box', drawn$layout$name))
  list(guide_boxes = sum(!vapply(drawn$grobs[boxes], inherits, logical(1), 'zeroGrob')),
   colour_breaks = if (is.null(colour)) list() else encode(colour$get_breaks()),
   colour_labels = if (is.null(colour)) list() else encode(colour$get_labels()),
   point_colours = encode(b$data[[1]]$colour),
   blank = inherits(helper$geom, 'GeomBlank'), inherit = helper$inherit.aes,
   helper_rows = nrow(helper$data), painted_rows = nrow(b$data[[1]]),
   blank_rows = nrow(b$data[[2]]),
   panels = lapply(b$layout$panel_params, function(p) list(x = encode(p$x$limits), y = encode(p$y$limits))),
   trained_colour = encode(b$plot$scales$get_scales('colour')$range$range))
 }), error = function(e) list(error = conditionMessage(e)))
 expansions[[length(expansions)+1L]] <- list(family = family, axes = axes, facet = facet,
  arguments = lapply(args, encode), result = result)
}
expansions <- c(Filter(function(x) x$axes != 'colour_shared', expansions),
 Filter(function(x) x$axes == 'colour_shared', expansions))
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', contracts = contracts,
 cases = cases, expansions = expansions), 'fixtures/parity/ggplot2/scale-limit-helpers.json',
 auto_unbox = TRUE, pretty = TRUE, digits = 15, null = 'null', na = 'null')
dev.off()
cat('captured', length(cases), 'typed limit and', length(expansions), 'expansion helper cases\n')
