# FIX-GG04: binned shape solid/default/NULL selection and theme precedence.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(unname(v), function(x) if (is.na(x)) NULL else x)
pdf(file = tempfile(fileext = '.pdf')); cases <- list()
for (route in c('default', 'solid', 'hollow', 'null'))
 for (theme_mode in c('absent', 'vector', 'count'))
  for (population in c('ordinary', 'constant', 'missing', 'empty')) {
   inputs <- switch(population, ordinary = c(0, 1, 4, 9), constant = c(4, 4),
                    missing = c(NA, 0, 4, NA, 9), empty = numeric())
   calls <- list()
   palette <- function(n) { calls[[length(calls)+1L]] <<- n; rep(3, if (theme_mode == 'vector') length(n) else n) }
   result <- tryCatch(suppressWarnings({
    args <- switch(route, default = list(), solid = list(solid = TRUE),
                   hollow = list(solid = FALSE), null = list(solid = NULL))
    p <- ggplot(data.frame(x = seq_along(inputs), v = inputs), aes(x, 1, shape = v)) +
         geom_point() + do.call(scale_shape_binned, args)
    if (theme_mode != 'absent') p <- p + theme(palette.shape.continuous = palette)
    b <- ggplot_build(p); grid::grid.draw(ggplotGrob(b))
    list(mapped = encode(b$data[[1]]$shape), guides = unname(lapply(b$plot$guides$params,
         function(g) list(values = encode(g$key$.value), labels = encode(g$key$.label),
                          mapped = encode(g$key$shape)))))
   }), error = function(e) list(error = conditionMessage(e)))
   cases[[length(cases)+1L]] <- list(route = route, theme_mode = theme_mode,
     population = population, inputs = encode(inputs), calls = as.list(calls), result = result)
  }
invisible(dev.off())
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases),
  'fixtures/parity/ggplot2/binned-style-defaults.json', auto_unbox = TRUE,
  pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('PASS', length(cases), 'binned shape default draws\n')
