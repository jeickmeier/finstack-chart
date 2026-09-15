# FIX-GG04: numeric constructor range/max_size forwarding and degenerate inputs.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(unname(v), function(x) if(is.na(x)) NULL else x)
pdf(file = tempfile('numeric-constructors-', fileext = '.pdf'))
cases <- list()
for(constructor in c('size', 'size_area', 'radius', 'alpha', 'linewidth', 'size_binned', 'alpha_binned', 'linewidth_binned', 'size_binned_area'))
for(configuration in c('default', 'custom', 'reversed', 'zero'))
for(population in c('ordinary', 'singleton', 'missing', 'all_missing', 'empty')) {
  channel <- if(constructor %in% c('size', 'size_area', 'radius', 'size_binned', 'size_binned_area')) 'size' else sub('_binned$', '', constructor)
  inputs <- switch(population, ordinary = c(-2, 0, 1, 4, 8), singleton = 2, missing = c(1, NA, 4), all_missing = NA_real_, empty = numeric())
  args <- if(configuration == 'default') list() else if(constructor %in% c('size_area', 'size_binned_area')) list(max_size = switch(configuration, custom = 10, reversed = -2, zero = 0)) else list(range = switch(configuration, custom = c(.2, .8), reversed = c(.8, .2), zero = c(0, 0)))
  result <- tryCatch(suppressWarnings({
    mapping <- aes(x, 1); mapping[[channel]] <- quote(v)
    if(channel == 'linewidth') { mapping$xend <- quote(x + .4); mapping$yend <- 1 }
    p <- ggplot(data.frame(x = seq_along(inputs), v = inputs), mapping) + (if(channel == 'linewidth') geom_segment() else geom_point()) + do.call(get(paste0('scale_', constructor)), c(args, list(guide = 'none')))
    b <- ggplot_build(p); g <- ggplotGrob(b); grid::grid.draw(g)
    panel <- g$grobs[[which(g$layout$name == 'panel')]]
    pts <- Filter(function(v) inherits(v, if(channel == 'linewidth') 'segments' else 'points'), panel$children)
    list(mapped = encode(b$data[[1]][[channel]]), mark_count = sum(vapply(pts, function(v) length(if(channel == 'linewidth') v$x0 else v$x), integer(1))), mark_colours = encode(unlist(lapply(pts, function(v) v$gp$col), use.names = FALSE)))
  }), error = function(e) list(error = conditionMessage(e)))
  cases[[length(cases) + 1L]] <- list(channel = channel, constructor = constructor, configuration = configuration, args = args, population = population, inputs = encode(inputs), result = result)
}
invisible(dev.off())
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases), 'fixtures/parity/ggplot2/numeric-paint-constructors.json', auto_unbox = TRUE, pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('PASS', length(cases), 'numeric constructor draws\n')
