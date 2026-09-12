# FIX-GG04 / GG2-03: positional null identity, factor levels and authored limits.
# Keep raw nullable keys separate from the displayed labels and numeric positions.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
populations <- list(mixed = c('b', NA, 'NA', 'a', NA),
                    finite = c('b', 'NA', 'a'),
                    missing = c(NA_character_, NA_character_),
                    empty = character())
limit_sets <- list(auto = NULL, first = c(NA, 'b', 'NA', 'a'),
                   middle = c('b', NA, 'NA', 'a'),
                   finite = c('b', 'NA', 'a'), only = NA_character_,
                   empty = character())
level_sets <- list(character = NULL, factor = c('b', 'NA', 'a', 'unused'),
                   nullable_factor = c('b', NA, 'NA', 'a', 'unused'))
break_sets <- list(auto = NULL, explicit = c(NA, 'NA', 'b', 'unused', 'a'))
cases <- list()
for (population in names(populations)) for (level_name in names(level_sets))
for (limit_name in names(limit_sets)) for (drop in c(FALSE, TRUE))
for (translate in c(FALSE, TRUE)) for (break_name in names(break_sets)) {
  values <- populations[[population]]
  levels <- level_sets[[level_name]]
  if (!is.null(levels)) values <- factor(values, levels = levels, exclude = NULL)
  args <- list(limits = limit_sets[[limit_name]], drop = drop, na.translate = translate)
  if (break_name == 'explicit') args$breaks <- break_sets[[break_name]]
  p <- ggplot(data.frame(x = values, y = seq_along(values)), aes(x, y)) +
    geom_point(na.rm = TRUE) + do.call(scale_x_discrete, args)
  result <- tryCatch(suppressWarnings({
    built <- ggplot_build(p)
    axis <- built$layout$panel_params[[1]]$x
    grob <- ggplotGrob(p)
    panel <- grob$grobs[[which(grob$layout$name == 'panel')]]
    point_count <- sum(vapply(panel$children, function(g)
      if (inherits(g, 'points')) length(g$x) else 0L, integer(1)))
    list(limits = as.list(unname(axis$get_limits())),
         breaks = as.list(unname(axis$get_breaks())),
         labels = as.list(unname(axis$get_labels())),
         range = lapply(unname(axis$continuous_range), function(v)
           if (is.infinite(v)) as.character(v) else v),
         major_positions = as.list(unname(axis$break_positions())),
         mapped = as.list(unname(built$data[[1]]$x)),
         point_positions = as.list(unname(axis$rescale(built$data[[1]]$x))),
         point_count = point_count)
  }), error = function(e) list(error = conditionMessage(e)))
  # The pinned reference rejects an explicit empty limit vector on a zero-row
  # position scale. Any other error is an oracle generation failure.
  expected_error <- population == 'empty' && limit_name == 'empty'
  stopifnot(is.null(result$error) != expected_error)
  if (expected_error) stopifnot(result$error == 'replacement has length zero')
  cases[[length(cases) + 1]] <- list(
    population = population, inputs = as.list(as.character(values)),
    level_name = level_name, levels = if (is.null(levels)) NULL else as.list(levels),
    limits_name = limit_name,
    limits = if (is.null(limit_sets[[limit_name]])) NULL else as.list(limit_sets[[limit_name]]),
    drop = drop, na_translate = translate, breaks_name = break_name,
    breaks = if (is.null(break_sets[[break_name]])) NULL else as.list(break_sets[[break_name]]),
    result = result)
}
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases),
  'fixtures/parity/ggplot2/position-null-categories.json', auto_unbox = TRUE,
  pretty = TRUE, digits = NA, null = 'null', na = 'null')
cat('PASS', length(cases), 'positional nullable category reference records\n')
