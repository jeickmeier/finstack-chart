# FIX-GG04: one scale trains and maps colour/fill from their joint population.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(unname(v), function(x) if(is.na(x)) NULL else x)
pdf(file = tempfile('shared-paint-', fileext = '.pdf'))
cases <- list()
for(family in c('continuous', 'binned', 'discrete', 'manual', 'identity'))
for(population in c('ordinary', 'missing', 'empty')) {
  numeric <- family %in% c('continuous', 'binned')
  colour <- if(numeric) c(0, 1, 2) else if(family == 'identity') c('#FF0000', '#008000', '#0000FF') else c('a', 'a', 'b')
  fill <- if(numeric) c(8, 10, 4) else if(family == 'identity') c('#FFFF00', '#00FFFF', '#FF00FF') else c('c', 'b', 'c')
  if(population == 'missing') { colour[2] <- NA; fill[3] <- NA }
  if(population == 'empty') { colour <- colour[FALSE]; fill <- fill[FALSE] }
  scale <- switch(family,
    continuous = scale_colour_gradient(aesthetics = c('colour', 'fill'), guide = 'none'),
    binned = scale_colour_steps(aesthetics = c('colour', 'fill'), guide = 'none'),
    discrete = scale_colour_hue(aesthetics = c('colour', 'fill'), guide = 'none'),
    manual = scale_colour_manual(aesthetics = c('colour', 'fill'), values = c('#FF0000', '#008000', '#0000FF'), guide = 'none'),
    identity = scale_colour_identity(aesthetics = c('colour', 'fill'), guide = 'none'))
  p <- ggplot(data.frame(x = seq_along(colour), colour, fill), aes(x, 1, colour = colour, fill = fill)) + geom_point(shape = 21, size = 4) + scale
  b <- suppressWarnings(ggplot_build(p)); g <- suppressWarnings(ggplotGrob(b)); grid::grid.draw(g)
  panel <- g$grobs[[which(g$layout$name == 'panel')]]
  pts <- Filter(function(v) inherits(v, 'points'), panel$children)
  cases[[length(cases) + 1L]] <- list(family = family, population = population,
    colour = encode(colour), fill = encode(fill),
    result = list(colour = encode(b$data[[1]]$colour), fill = encode(b$data[[1]]$fill),
      mark_count = sum(vapply(pts, function(v) length(v$x), integer(1)))))
}
invisible(dev.off())
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases), 'fixtures/parity/ggplot2/shared-paint-aesthetics.json', auto_unbox = TRUE, pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('PASS', length(cases), 'joint colour/fill reference draws\n')
