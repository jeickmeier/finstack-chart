# GG-00: complete release exports and inherited ggproto argument/default inventory.
stopifnot(as.character(getRversion()) == "4.6.1", as.character(packageVersion("ggplot2")) == "4.0.3")
options(repos = c(CRAN = "https://cran.r-project.org"), width = 120)
suppressPackageStartupMessages(library(ggplot2))
args <- commandArgs(trailingOnly = TRUE)
out <- if (length(args)) args[[1]] else "fixtures/parity/ggplot2"
dir.create(out, recursive = TRUE, showWarnings = FALSE)
write_json <- function(x, file) jsonlite::write_json(x, file.path(out, file), auto_unbox = TRUE,
                                                   pretty = TRUE, null = "null", digits = NA)
owner <- function(name) {
  key <- gsub("_", "", tolower(name))
  has <- function(pattern) grepl(pattern, key)
  if (has("^(coordsf|geomsf|statsf|annotationmap|coordmap|geommap|mapdata)")) return("GG-15")
  if (has("^coord")) return("GG-13")
  if (has("^(facet|labeller|label)")) return("GG-12")
  if (has("^(guide|drawkey)")) return("GG-05")
  if (has("^(scale|continuousscale|discretescale|binnedscale|expandlimits|expansion|lims|[xy]lim|dupaxis|secaxis)")) return("GG-04")
  if (has("^(theme|element|rel$|margin|calcelement)")) return("GG-14")
  if (has("(smooth|quantile)")) return("GG-10")
  if (has("(bin2d|binhex|density2d|contour|ellipse|summary2d|summaryhex)")) return("GG-11")
  if (has("(boxplot|density|violin|dotplot|ecdf|qq|rug|ydensity)")) return("GG-09")
  if (has("^(stat|position|meancl|meansdl|meanse|medianhilow)")) return("GG-06")
  if (has("(geomtext|geomlabel|annotationcustom|annotate)")) return("GG-08")
  if (has("^geom")) return("GG-07")
  if (has("^(ggsave|ggplotgtable|ggplotgrob)")) return("GG-17")
  if (has("^(aes|after|stage$|i$|vars$|layer$|qplot|quickplot|ggplot$|ggplotbuild|ggplotadd)")) return("GG-02")
  "GG-16"
}

defaults <- function(f) {
  if (!is.function(f)) return(NULL)
  if (inherits(f, "ggproto_method")) f <- environment(f)$f
  x <- formals(f)
  if (is.null(x)) return(list())
  lapply(as.list(x), function(v) paste(deparse(v, width.cutoff = 500L), collapse = "\n"))
}
literal <- function(x) {
  if (is.null(x)) return(NULL)
  if (is.function(x)) return(list(kind = "function", arguments = defaults(x)))
  if (inherits(x, "quosure")) return(paste(deparse(rlang::quo_get_expr(x)), collapse = "\n"))
  if (is.language(x)) return(paste(deparse(x), collapse = "\n"))
  if (is.atomic(x)) return(unname(x))
  if (is.list(x) && !is.environment(x)) return(lapply(x, literal))
  list(kind = class(x)[[1]])
}
ns <- asNamespace("ggplot2")
exports <- sort(getNamespaceExports("ggplot2"))
rows <- lapply(exports, function(name) {
  x <- getExportedValue("ggplot2", name)
  row <- list(id = paste0("ggplot2/4.0.3/", name), name = name, owner = owner(name),
              requirement = "GG2-01/GG2-12", status = "OPEN", class = I(class(x)), arguments = defaults(x))
  if (inherits(x, "ggproto")) {
    resolved <- as.list(x)
    row$inherited_members <- lapply(resolved[sort(names(resolved))], literal)
  }
  row
})
# Retain complete reference usage/default/computed-variable text, including aliases.
# This is release-owned documentation, not a guessed generated-field list.
rd <- tools::Rd_db("ggplot2")
computed_items <- function(x) {
  if (!is.list(x)) return(list())
  if (identical(attr(x,"Rd_tag"),"\\item") && length(x)>=2) return(list(list(
    field = paste(unlist(x[[1]]),collapse=""),
    description = paste(unlist(x[[2]]),collapse=""))))
  unname(unlist(lapply(x,computed_items),recursive=FALSE))
}
docs <- lapply(sort(names(rd)), function(name) {
  content <- rd[[name]]
  tags <- vapply(content, function(x) { t <- attr(x,"Rd_tag"); if(is.null(t)) "" else t }, "")
  aliases <- unname(vapply(content[tags == "\\alias"], function(x) paste(unlist(x),collapse=""), ""))
  sections <- content[tags == "\\section"]
  computed <- sections[vapply(sections,function(x) grepl("computed variables",paste(unlist(x[[1]]),collapse=""),ignore.case=TRUE),TRUE)]
  computed_text <- paste(unlist(computed),collapse="")
  expressions <- unique(regmatches(computed_text,gregexpr("after_stat\\([^)]*\\)",computed_text))[[1]])
  fields <- c(computed_items(computed),lapply(expressions,function(expression)list(field=expression)))
  list(file = name, aliases = I(aliases), computed_fields = as.list(fields),
       text = gsub(".\b","",paste(capture.output(tools::Rd2txt(content)), collapse = "\n")))
})
rows <- lapply(rows,function(row) {
  normalized <- function(x) tolower(gsub("_","",x))
  matched <- docs[vapply(docs,function(doc) normalized(row$name) %in% normalized(doc$aliases),TRUE)]
  row$documentation <- I(unname(vapply(matched,function(doc)doc$file,"")))
  row$computed_fields <- unname(as.list(unlist(lapply(matched,function(doc)doc$computed_fields),recursive=FALSE)))
  row
})
write_json(list(schema_version = 1, reference = "ggplot2 4.0.3", export_count = length(rows), exports = rows), "inventory.json")
write_json(list(schema_version = 1, documentation = docs), "documentation.json")
writeLines(capture.output(sessionInfo()), file.path(out,"session-info.txt"))
renv::snapshot(project = "tools/reference/r", library = .libPaths(),
               lockfile = "tools/reference/r/renv.lock", type = "all", prompt = FALSE)
cat("PASS GG-00 inventory:", length(rows), "exports and", length(docs), "release documentation records; capability rows remain OPEN.\n")
