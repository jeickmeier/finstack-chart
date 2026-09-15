# FIX-GG04: population-dependent discrete position palettes and numeric rejection.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(unname(v), function(x) if (is.na(x)) NULL else if (is.numeric(x) && is.infinite(x)) { if (x > 0) "Infinity" else "-Infinity" } else x)
cases <- list()
pdf(file = tempfile(fileext = '.pdf'))
for (route in c('reverse', 'spread', 'short', 'named', 'missing', 'character', 'null'))
 for (population in c('ordinary', 'constant', 'missing', 'empty'))
  for (limit_mode in c('automatic', 'retained', 'empty')) {
   inputs <- switch(population, ordinary = c('c', 'a', 'b', 'a'), constant = c('b', 'b'),
                    missing = c(NA, 'a', 'c', NA), empty = character())
   calls <- integer()
   palette <- function(n) {
    calls <<- c(calls, n)
    switch(route, reverse = rev(seq_len(n)), spread = seq_len(n)^2,
     short = seq_len(max(0, n - 1)), named = setNames(seq_len(n)^2, rev(letters[seq_len(n)])),
     missing = rep(NA_real_, n), character = as.character(seq_len(n)), null = NULL)
   }
   limits <- switch(limit_mode, automatic = NULL, retained = c('c', 'b', 'a', 'd'), empty = character())
   result <- tryCatch(suppressWarnings({
    p <- ggplot(data.frame(x = inputs, y = seq_along(inputs)), aes(x, y)) + geom_point() +
      scale_x_discrete(palette = palette, limits = limits)
    b <- ggplot_build(p)
    grid::grid.draw(ggplotGrob(b))
    a <- b$layout$panel_params[[1]]$x
    list(mapped = encode(b$data[[1]]$x), limits = encode(a$get_limits()),
         range = encode(a$continuous_range), breaks = encode(a$get_breaks()),
         positions = encode(a$rescale(b$data[[1]]$x)),
         major_positions = encode(a$break_positions()), labels = encode(a$get_labels()))
   }), error = function(e) list(error = conditionMessage(e)))
   cases[[length(cases)+1L]] <- list(route = route, population = population,
    limit_mode = limit_mode, inputs = encode(inputs), calls = as.list(calls), result = result)
  }
invisible(dev.off())
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases),
 'fixtures/parity/ggplot2/positional-palette-functions.json', auto_unbox = TRUE,
 pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('PASS', length(cases), 'discrete positional palette draws\n')
