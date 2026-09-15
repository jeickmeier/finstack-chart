# FIX-GG04: non-color discrete label callbacks through actual guide preparation.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x), function(v) if (is.na(v)) NULL else v)
cases <- list()
for (channel in c('size', 'alpha', 'linewidth', 'shape', 'linetype'))
 for (population in c('ordinary', 'nullable', 'all_missing', 'empty'))
  for (limit_mode in c('trained', 'explicit'))
   for (break_mode in c('auto', 'explicit', 'named', 'empty'))
    for (mode in c('indexed', 'missing', 'short', 'empty', 'named')) {
     values <- switch(population, ordinary = c('a', 'b', 'c'),
                      nullable = c('a', 'b', NA), all_missing = c(NA_character_, NA_character_),
                      empty = character())
     calls <- list()
     label_function <- function(x) {
      calls[[length(calls) + 1L]] <<- list(values = encode(x), names = as.list(names(x)))
      labels <- paste0(seq_along(x), '/', length(x))
      switch(mode, indexed = labels,
             missing = replace(labels, seq_along(x) %% 2 == 0, NA_character_),
             short = head(labels, 1), empty = character(),
             named = setNames(labels, rev(as.character(x))))
     }
     result <- tryCatch(suppressWarnings({
      constructor <- get(paste0('scale_', channel, '_discrete'))
      scale <- constructor(limits = if (limit_mode == 'explicit') c('c', 'b', 'a', NA) else NULL,
       breaks = switch(break_mode, auto = waiver(), explicit = c('z', 'b', 'b', NA, 'a'),
                       named = c(Z = 'z', B = 'b', B2 = 'b', M = NA, A = 'a'), empty = character()),
       labels = label_function)
      data <- data.frame(x = seq_along(values), y = seq_along(values), v = values)
      mapping <- aes(x, y, group = 1)
      mapping[[channel]] <- quote(v)
      geometry <- if (channel %in% c('linewidth', 'linetype')) geom_line() else geom_point()
      built <- ggplot_build(ggplot(data, mapping) + geometry + scale)
      keys <- lapply(built$plot$guides$params, function(g)
       list(values = encode(g$key$.value), labels = encode(g$key$.label)))
      list(keys = unname(keys))
     }), error = function(e) list(error = conditionMessage(e)))
     cases[[length(cases) + 1L]] <- list(channel = channel, population = population,
      limit_mode = limit_mode, break_mode = break_mode, label_mode = mode,
      inputs = encode(values), calls = calls, result = result)
    }
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases),
 'fixtures/parity/ggplot2/noncolor-discrete-label-functions.json',
 auto_unbox = TRUE, pretty = TRUE, digits = NA, null = 'null')
cat('PASS', length(cases), 'non-color discrete label callback records\n')
