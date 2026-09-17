# Source-owned GG13 nonlinear coordinate compatibility boundary.
library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
source_text=paste(deparse(environment(GeomDotplot$draw_group)$f),collapse='\n')
stopifnot(grepl('does not work properly with non-linear coordinates',source_text,fixed=TRUE))
jsonlite::write_json(list(R=as.character(getRversion()),ggplot2=as.character(packageVersion('ggplot2')),draw_group=source_text), 'fixtures/parity/ggplot2/dotplot-coordinate-controls.json',pretty=TRUE,auto_unbox=TRUE)
