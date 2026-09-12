# FIX-GG04: independent asymmetric and constant-range expansion oracle.
stopifnot(as.character(packageVersion('ggplot2')) == '4.0.3',
          as.character(packageVersion('scales')) == '1.4.0')
cases <- list()
for (domain in list(c(0, 1), c(-3, 17), c(0, 0), c(10, 10),
                    c(10, 0), c(1e-20, 2e-20), c(1e12, 1e12 + 100),
                    c(1, 1 + 1e-14), c(-1, -1 + 1e-14), c(0, 1e-300))) {
  for (mult in list(c(0, 0), c(.05, .05), c(.1, .2), c(-.1, .2))) {
    for (add in list(c(0, 0), c(1, 2), c(-1, .5))) {
      cases[[length(cases) + 1]] <- list(
        domain = domain, mult = mult, add = add,
        expanded = ggplot2:::expand_range4(domain, ggplot2::expansion(mult, add)))
    }
  }
}
panels <- lapply(list(c(0, 1), c(10, 10), c(0, 0), c(-3, 17)), function(x) {
  p <- ggplot2::ggplot(data.frame(x = x, y = c(0, 1)), ggplot2::aes(x, y)) + ggplot2::geom_point()
  axis <- ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x
  keep <- !is.na(axis$breaks)
  list(domain = x, kind = 'auto', explicit = FALSE,
       breaks = as.list(axis$breaks[keep]), labels = as.list(axis$get_labels()[keep]),
       expanded = ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x.range)
})
for (kind in c('linear', 'log')) for (x in list(c(1, 100), c(10, 10), c(100, 1))) {
  for (explicit in c(FALSE, TRUE)) for (custom in c(FALSE, TRUE)) {
    mult <- if(custom) c(.1, .2) else c(.05, .05)
    add <- if(custom) c(.3, .4) else c(0, 0)
    scale <- if(kind == 'log') ggplot2::scale_x_log10 else ggplot2::scale_x_continuous
    p <- ggplot2::ggplot(data.frame(x = x, y = c(0, 1)), ggplot2::aes(x, y)) +
      ggplot2::geom_point() + scale(limits = if(explicit) x else NULL,
        expand = ggplot2::expansion(mult, add))
    panels[[length(panels) + 1]] <- list(domain = x, kind = kind, explicit = explicit,
      custom = custom, mult = mult, add = add,
      expanded = ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x.range)
    axis <- ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x
    keep <- !is.na(axis$breaks)
    panels[[length(panels)]]$breaks <- as.list(if(kind == 'log') 10^axis$breaks[keep] else axis$breaks[keep])
    panels[[length(panels)]]$labels <- as.list(axis$get_labels()[keep])
  }
}
for (kind in c('linear', 'log')) for (explicit in c(FALSE, TRUE)) {
  x <- c(10, 10)
  scale <- if(kind == 'log') ggplot2::scale_x_log10 else ggplot2::scale_x_continuous
  p <- ggplot2::ggplot(data.frame(x = x, y = c(0, 1)), ggplot2::aes(x, y)) +
    ggplot2::geom_point() + scale(limits = if(explicit) x else NULL,
      expand = ggplot2::expansion(0, 0))
  build <- ggplot2::ggplot_build(p)
  panel <- build$layout$panel_params[[1]]
  axis <- panel$x
  keep <- !is.na(axis$breaks)
  panels[[length(panels) + 1]] <- list(domain = x, kind = kind, explicit = explicit,
    custom = TRUE, mult = c(0, 0), add = c(0, 0), expanded = panel$x.range,
    breaks = as.list(if(kind == 'log') 10^axis$breaks[keep] else axis$breaks[keep]),
    labels = as.list(axis$get_labels()[keep]),
    projected = build$layout$coord$transform(build$data[[1]], panel)$x)
}
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / scales 1.4.0', cases = cases, panels = panels),
  'fixtures/parity/ggplot2/expansion.json', auto_unbox = TRUE, pretty = TRUE, digits = NA)
cat('PASS', length(cases), 'range expansion records\n')
