# FIX-GG04: public reference alpha conversion at every byte midpoint.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3',
          as.character(packageVersion('scales')) == '1.4.0')
cases <- lapply(0:254, function(i) {
  values <- (i + .5) / 255 + c(-1e-15, 0, 1e-15)
  list(values = sprintf("%.17g", values), colours = scales::alpha('#000000', values))
})
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1', cases = cases),
  'fixtures/parity/ggplot2/alpha-byte-rounding.json', auto_unbox = TRUE, pretty = TRUE, digits = NA)
cat('captured', length(cases)*3L, 'alpha byte boundary cases\n')
