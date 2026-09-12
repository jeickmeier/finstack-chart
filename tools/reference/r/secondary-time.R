# FIX-GG04: secondary Date/datetime guides preserve their calendar representation.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
for (kind in c('datetime', 'date')) {
 for (span in if (kind == 'date') list(c(0, 20), c(-40, 400), c(18000, 18365)) else list(c(0, 120), c(0, 90000), c(1710046800, 1710133200), c(1730606400, 1730692800))) {
  for (zone in if(kind == 'datetime' && span[1] > 1700000000) c('UTC','America/New_York') else 'UTC') {
  for (offset in if (kind == 'date') c(-7, 0, 30) else c(-3600, 0, 19800)) {
   for (expand in c(FALSE, TRUE)) {
    limits <- if(kind == 'date') as.Date(span, origin='1970-01-01') else as.POSIXct(span, origin='1970-01-01', tz=zone)
    secondary <- ggplot2::sec_axis(function(x) x + offset)
    scale <- if(kind == 'date') ggplot2::scale_x_date else ggplot2::scale_x_datetime
    p <- ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y)) + ggplot2::geom_point() +
      scale(limits=limits, expand=ggplot2::expansion(mult=if(expand) .05 else 0),sec.axis=secondary)
    panel <- ggplot2::ggplot_build(p)$layout$panel_params[[1]]
    info <- panel$x.sec$break_info
    order <- order(info$major)
    cases[[length(cases)+1]] <- list(kind=kind,zone=zone,limits=span,offset=offset,expand=expand,
      range=info$range,values=as.list(as.numeric(info$major_source_user[order])),
      labels=as.list(info$labels[order]),positions=unname(as.list(info$major[order])))
   }
  }
 }
}
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),
 'fixtures/parity/ggplot2/secondary-time.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'secondary time records\n')
