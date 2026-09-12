# FIX-GG04: discrete shape and linetype overflow, independently captured from scales.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('scales')) == '1.4.0')
cases <- list()
for (kind in c('solid', 'hollow', 'linetype')) for (n in c(0,1,6,7,13,14)) {
  palette <- switch(kind, solid=scales::pal_shape(TRUE), hollow=scales::pal_shape(FALSE), linetype=scales::pal_linetype())
  warnings <- character()
  values <- withCallingHandlers(palette(n), warning=function(w) { warnings <<- c(warnings, conditionMessage(w)); invokeRestart('muffleWarning') })
  cases[[length(cases)+1]] <- list(kind=kind, n=n, values=as.list(values), warnings=as.list(warnings))
}
jsonlite::write_json(list(reference='scales 1.4.0',cases=cases), 'fixtures/parity/ggplot2/style-palettes.json', auto_unbox=TRUE, pretty=TRUE, na='null')
cat('PASS',length(cases),'style palette records\n')
