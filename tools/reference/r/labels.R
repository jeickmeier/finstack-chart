# FIX-GG04: ggplot transform-default labels use vector-wide base R formatting.
stopifnot(as.character(packageVersion('scales')) == '1.4.0')
inputs <- list(numeric(), c(0), c(-0, 0), c(.1, .2), c(1, 1.01),
  c(1e6, 1.1e6), c(1e-6, 1), c(-1, 1e6), c(-1, 1e5),
  c(1e-8, 1.234567891), c(1.2345678, 1.234567891), c(1, 1e100),
  c(1e20, 1e-20), c(-1e-300, 0, 1e300), c(9.9999999, 10.000001))
breaks <- jsonlite::read_json('fixtures/parity/ggplot2/breaks.json')$cases
for (case in breaks) inputs[[length(inputs) + 1]] <- unlist(case$breaks)
set.seed(1944)
for(i in 1:200) {
  inputs[[length(inputs) + 1]] <- runif(sample(1:8, 1), -10, 10) * 10^sample(-20:20, 1)
}
cases <- lapply(inputs, function(x) list(values = as.list(x),
  labels = as.list(scales::format_format()(x))))
jsonlite::write_json(list(reference = 'scales 1.4.0 / R 4.6.1 r90187', cases = cases),
  'fixtures/parity/ggplot2/labels.json', auto_unbox = TRUE, pretty = TRUE, digits = NA)
cat('PASS', length(cases), 'label vectors\n')
