# GG-04: immutable source bodies for exhaustive forwarding and rejection reconciliation.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
index <- jsonlite::read_json('docs/evidence/ggplot-scales-coverage.json')
function_contract <- function(value) {
  # ggproto method access binds self around the original method.
  if (inherits(value, 'ggproto_method')) value <- environment(value)$f
  list(formals = lapply(formals(value), function(x) paste(deparse(x, width.cutoff = 500L), collapse = '\n')),
       body = paste(deparse(body(value), width.cutoff = 500L), collapse = '\n'))
}
contracts <- lapply(index$exports, function(entry) {
  value <- getExportedValue('ggplot2', entry$name)
  if (is.function(value)) return(c(list(name = entry$name, kind = 'function'), function_contract(value)))
  stopifnot(inherits(value, 'ggproto'))
  fields <- as.list(value)
  methods <- names(fields)[vapply(fields, is.function, logical(1))]
  list(name = entry$name, kind = 'ggproto', classes = as.list(class(value)),
       methods = setNames(lapply(methods, function(name) function_contract(value[[name]])), methods),
       fields = as.list(setdiff(names(fields), methods)))
})
stopifnot(length(contracts) == 152L)
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1', contracts = contracts),
  'fixtures/parity/ggplot2/scale-constructor-contracts.json', pretty = TRUE, auto_unbox = TRUE)
cat('captured', length(contracts), 'constructor and class source contracts\n')
