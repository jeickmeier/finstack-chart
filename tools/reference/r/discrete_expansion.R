# FIX-GG04: discrete index expansion and category projection.
stopifnot(as.character(packageVersion('ggplot2')) == '4.0.3')
policies <- list(ggplot2::expansion(add = .6), ggplot2::expansion(mult = c(.1, .2), add = c(.3, .7)), ggplot2::expansion(0, 0))
cases <- panels <- list()
for(n in c(0, 1, 2, 5)) for(expand in policies) {
  labels <- letters[seq_len(n)]
  for(continuous in list(NULL, c(.1, 7.2), c(2, 2))) {
    cases[[length(cases) + 1]] <- list(count = n, mult = expand[c(1, 3)], add = expand[c(2, 4)], continuous = continuous,
      expanded = ggplot2:::expand_limits_discrete(if(n == 0) NULL else labels, expand, range_continuous = continuous))
  }
  if(n == 0) next
  p <- ggplot2::ggplot(data.frame(x = labels, y = seq_len(n)), ggplot2::aes(x, y)) + ggplot2::geom_point() + ggplot2::scale_x_discrete(expand = expand)
  build <- ggplot2::ggplot_build(p)
  panel <- build$layout$panel_params[[1]]
  panels[[length(panels) + 1]] <- list(labels = as.list(labels), mult = expand[c(1, 3)], add = expand[c(2, 4)], expanded = panel$x.range,
    projected = as.list(build$layout$coord$transform(build$data[[1]], panel)$x))
}
jsonlite::write_json(list(cases = cases, panels = panels), 'fixtures/parity/ggplot2/discrete_expansion.json', auto_unbox = TRUE, pretty = TRUE, digits = NA)
cat('PASS', length(cases), 'discrete ranges and', length(panels), 'panels\n')
