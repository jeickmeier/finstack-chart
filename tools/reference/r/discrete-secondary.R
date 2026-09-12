# FIX-GG04 / GG2-03: discrete secondary guides inherit keys but expose numeric positions.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(unname(v), function(x)
  if (is.na(x)) NULL else if (is.infinite(x)) if (x > 0) 'Infinity' else '-Infinity' else x)
populations <- list(mixed = c('b', NA, 'NA', 'a'), finite = c('b', 'NA', 'a'),
                    missing = c(NA_character_, NA_character_), empty = character())
limits <- list(auto = NULL, authored = c(NA, 'b', 'NA', 'a'))
controls <- list(
  inherit = list(), hidden = list(labels = NULL), empty = list(breaks = NULL),
  numeric = list(breaks = c(-1, 0, .5, 1, 1.5, 2, 3, 4, 5, NA, Inf)),
  character = list(breaks = c('b', NA, 'NA', 'absent', 'a')),
  explicit = list(breaks = c('b', NA, 'NA', 'a'), labels = c('Bee', 'Missing', 'Literal', 'A')),
  bad_labels = list(breaks = c('b','a'), labels = 'wrong'),
  transformed = list(transform = function(x) x * 2),
  primary_breaks = list(), primary_labels = list(), primary_hidden = list())
cases <- list()
for (population in names(populations)) for (limit_name in names(limits))
for (translate in c(FALSE, TRUE)) for (control in names(controls)) {
  values <- populations[[population]]; args <- controls[[control]]
  result <- tryCatch(suppressWarnings({
    primary_args <- list(limits = limits[[limit_name]], na.translate = translate, sec.axis = do.call(dup_axis, args))
    if (control %in% c('primary_breaks','primary_labels')) primary_args$breaks <- c('b','a')
    if (control == 'primary_labels') primary_args$labels <- c('Bee','Aye')
    if (control == 'primary_hidden') primary_args['labels'] <- list(NULL)
    p <- ggplot(data.frame(x = values, y = seq_along(values)), aes(x,y)) + geom_point() +
      do.call(scale_x_discrete, primary_args)
    b <- ggplot_build(p); primary <- b$layout$panel_params[[1]]$x
    secondary <- b$layout$panel_params[[1]]$x.sec
    list(range = encode(secondary$get_limits()), breaks = encode(secondary$get_breaks()),
         labels = as.list(unname(secondary$get_labels())), positions = encode(secondary$break_positions()),
         primary_limits = as.list(unname(primary$get_limits())),
         primary_labels = as.list(unname(primary$get_labels())),
         point_positions = encode(primary$rescale(b$data[[1]]$x)))
  }), error = function(e) list(error = conditionMessage(e)))
  cases[[length(cases) + 1]] <- list(population = population, inputs = as.list(values),
    limits_name = limit_name, limits = if(is.null(limits[[limit_name]])) NULL else as.list(limits[[limit_name]]),
    na_translate = translate, control = control, result = result)
}
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', cases = cases),
  'fixtures/parity/ggplot2/discrete-secondary.json', auto_unbox = TRUE,
  pretty = TRUE, digits = 17, null = 'null', na = 'null')
cat('PASS', length(cases), 'discrete secondary axis reference panels\n')
