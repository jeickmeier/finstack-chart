# FIX-GG04: secondary break/label callbacks retain transformed domains and time classes.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
Sys.setenv(TZ = 'UTC')
encode <- function(v) {
 if (inherits(v, c('Date', 'POSIXt'))) v <- as.numeric(v)
 lapply(unname(v), function(x) if (is.na(x)) NULL else if (is.numeric(x) && is.infinite(x)) {
  if (x > 0) 'Infinity' else '-Infinity'
 } else x)
}
describe <- function(v) list(values = encode(v), names = as.list(names(v)),
 classes = as.list(class(v)), timezone = as.list(attr(v, 'tzone')))
cases <- list(); pdf(file = tempfile(fileext = '.pdf'))
for (family in c('numeric', 'discrete', 'date', 'datetime'))
 for (conversion in if (family == 'numeric') c('identity', 'affine', 'square') else if (family == 'discrete') 'identity' else c('identity', 'shift'))
  for (population in c('ordinary', 'constant', 'missing', 'empty'))
   for (mode in if (family %in% c('date', 'datetime')) c('domain', 'mixed', 'empty', 'typed_empty', 'null') else c('domain', 'mixed', 'empty', 'null'))
    for (label_mode in c('default', 'function'))
     for (expand in if (population == 'constant' && conversion == 'identity' && mode == 'domain' && label_mode == 'function') c(TRUE, FALSE) else TRUE) {
     raw <- switch(population, ordinary = c(1, 4, 10), constant = c(4, 4),
                   missing = c(NA, 1, 10), empty = numeric())
     values <- switch(family, numeric = raw, discrete = ifelse(is.na(raw), NA_character_, as.character(raw)),
                      date = as.Date(raw, origin = '1970-01-01'),
                      datetime = as.POSIXct(raw, origin = '1970-01-01', tz = 'UTC'))
     calls <- list(); label_calls <- list()
     breaks <- function(x) {
      calls[[length(calls) + 1L]] <<- describe(x)
      switch(mode, domain = x, mixed = setNames(c(x[2], mean(x), x[1], x[1], NA, Inf, -Inf),
          c('last', 'middle', 'first', 'again', 'missing', 'positive', 'negative')),
          empty = numeric(), typed_empty = x[FALSE], null = NULL)
     }
     labels <- function(x) {
      label_calls[[length(label_calls) + 1L]] <<- describe(x)
      paste0(seq_along(x), '/', length(x))
     }
     result <- tryCatch(suppressWarnings({
      conversion_fn <- switch(conversion, identity = identity, affine = function(x) 2 * x + 3,
                              square = function(x) x^2, shift = function(x) x + 2)
      secondary <- dup_axis(transform = conversion_fn, breaks = breaks,
                            labels = if (label_mode == 'function') labels else waiver())
      scale <- switch(family, numeric = scale_x_continuous, discrete = scale_x_discrete,
                      date = scale_x_date, datetime = scale_x_datetime)
      b <- ggplot_build(ggplot(data.frame(x = values, y = rep(1, length(values))), aes(x, y)) +
                        geom_point() + (if (family == 'datetime') scale(sec.axis = secondary, timezone = 'UTC', expand = if (expand) waiver() else expansion(0)) else scale(sec.axis = secondary, expand = if (expand) waiver() else expansion(0))))
      grid::grid.draw(ggplotGrob(b))
      secondary <- b$layout$panel_params[[1]]$x.sec
      list(breaks = encode(secondary$get_breaks()), labels = encode(secondary$get_labels()),
           positions = encode(secondary$break_positions()), range = encode(secondary$get_limits()),
           user_values = encode(secondary$break_info$major_source_user))
     }), error = function(e) list(error = conditionMessage(e)))
     cases[[length(cases) + 1L]] <- list(family = family, conversion = conversion,
      population = population, mode = mode, label_mode = label_mode, expand = expand, inputs = encode(values),
      calls = calls, label_calls = label_calls, result = result)
    }
invisible(dev.off())
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', timezone_resource = 'UTC', cases = cases),
 'fixtures/parity/ggplot2/secondary-guide-functions.json', auto_unbox = TRUE,
 pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('PASS', length(cases), 'secondary guide callback draws\n')
