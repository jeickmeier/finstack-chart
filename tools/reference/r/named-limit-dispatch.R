# GG2-03/FIX-GG04: named lims dispatch versus its selected public constructor.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x), function(v) {
  if (is.na(v)) NULL else if (is.infinite(v)) {
    if (v > 0) 'Infinity' else '-Infinity'
  } else v
})
convert <- function(x, family) switch(family,
  numeric = as.numeric(x), character = as.character(x),
  factor = factor(as.character(x), levels = c('5', '4', '2', '1', '0')),
  date = as.Date(x, origin = '1970-01-01'),
  datetime = as.POSIXct(x * 86400, origin = '1970-01-01', tz = 'UTC'))
inspect <- function(helper, channel, inputs) {
  d <- data.frame(v = inputs, i = seq_along(inputs))
  mapping <- aes(i, i)
  mapping[[channel]] <- rlang::quo(v)
  b <- suppressWarnings(ggplot_build(ggplot(d, mapping) + geom_point() + helper))
  resolved <- b$plot$scales$get_scales(channel)
  positional <- channel %in% c('x', 'y')
  if (positional) {
    p <- b$layout$panel_params[[1]][[channel]]
    return(list(scale_class = as.list(class(helper)), transform = helper$get_transformation()$name,
      mapped = encode(as.numeric(b$data[[1]][[channel]])), limits = encode(p$limits),
      breaks = encode(p$breaks), labels = encode(p$get_labels()),
      point_positions = encode(p$rescale(b$data[[1]][[channel]])),
      break_positions = encode(p$break_positions())))
  }
  list(scale_class = as.list(class(helper)), aesthetics = as.list(helper$aesthetics),
       limits = encode(resolved$get_limits()), breaks = encode(resolved$get_breaks()),
       labels = encode(resolved$get_labels()), mapped = encode(b$data[[1]][[channel]]))
}
cases <- list(); positional_cases <- list()
for (channel in c('x', 'y', 'colour', 'color', 'fill', 'size', 'alpha', 'shape', 'linewidth', 'linetype'))
 for (family in c('numeric', 'character', 'factor', 'date', 'datetime'))
  for (mode in c('ascending', 'descending', 'lower_missing', 'both_missing', 'single', 'empty')) {
    values <- switch(mode, ascending = c(1, 4), descending = c(4, 1),
      lower_missing = c(NA, 4), both_missing = c(NA, NA), single = 2, empty = numeric())
    authored <- convert(values, family); inputs <- convert(c(0, 1, 2, 4, 5), family)
    type <- switch(family, numeric = 'continuous', character = 'discrete', factor = 'discrete',
                   date = 'date', datetime = 'datetime')
    constructor <- paste('scale', channel, type, sep = '_')
    run <- function(named) tryCatch(suppressWarnings({
      helper <- if (named) do.call(lims, setNames(list(authored), channel))[[1]] else {
        # Public numeric/temporal helper arity is checked before constructor dispatch.
        if (family %in% c('numeric', 'date', 'datetime') && length(authored) != 2L)
          stop('invalid helper endpoint count')
        args <- list(limits = if (family == 'factor') as.character(authored) else authored)
        if (family == 'numeric') args$transform <- if (!anyNA(authored) && authored[1] > authored[2]) 'reverse' else 'identity'
        do.call(get(constructor, asNamespace('ggplot2')), args)
      }
      inspect(helper, if (channel == 'color') 'colour' else channel, inputs)
    }), error = function(e) list(error = conditionMessage(e)))
    named <- run(TRUE); direct <- run(FALSE)
    equal <- if (!is.null(named$error)) !is.null(direct$error) else identical(named, direct)
    stopifnot(equal)
    cases[[length(cases) + 1L]] <- list(channel = channel, family = family, mode = mode,
      constructor = constructor, result = named, direct_result = direct, equivalent = equal)
    if (channel %in% c('x', 'y') && family != 'factor')
      positional_cases[[length(positional_cases) + 1L]] <- list(axis = channel, family = family,
        mode = mode, inputs = encode(c(0, 1, 2, 4, 5)), authored = encode(values), result = named)
  }
# Verify composition keeps ordinary list order, and duplicate axes use last-scale wins.
composition <- lapply(list(list(x = c(1, 4), y = c(4, 1)),
                          setNames(list(c(1, 4), c(2, 5)), c('x', 'x'))), function(args) {
  helpers <- do.call(lims, args)
  p <- ggplot(data.frame(x = 0:5, y = 0:5), aes(x, y)) + geom_point()
  named <- suppressMessages(ggplot_build(p + helpers))
  direct <- p
  for (i in seq_along(args)) direct <- suppressMessages(direct + do.call(get(paste0(names(args)[i], 'lim')), list(args[[i]])))
  direct <- suppressMessages(ggplot_build(direct))
  stopifnot(identical(named$data, direct$data))
  list(names = as.list(names(args)), mapped_x = encode(named$data[[1]]$x), mapped_y = encode(named$data[[1]]$y))
})
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1',
  cases = positional_cases, dispatch = cases, composition = composition),
  'fixtures/parity/ggplot2/named-limit-dispatch.json', auto_unbox = TRUE,
  pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('PASS', length(cases), 'named/public constructor comparisons;', length(positional_cases), 'positional outcomes; 2 compositions\n')
