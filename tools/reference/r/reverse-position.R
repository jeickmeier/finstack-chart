# FIX-GG04: reverse positional transforms, limits and panel controls.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
for (x in list(c(-3, 0, 17), c(17, 0, -3), c(10, 10, 10), c(0, 0, 0))) {
  for (explicit in c(FALSE, TRUE)) for (custom in c(FALSE, TRUE)) {
    mult <- if(custom) c(.1, .2) else c(.05, .05)
    add <- if(custom) c(.3, .4) else c(0, 0)
    p <- ggplot2::ggplot(data.frame(x=x,y=seq_along(x)),ggplot2::aes(x,y)) +
      ggplot2::geom_point() + ggplot2::scale_x_reverse(limits=if(explicit)range(x)else NULL,
        expand=ggplot2::expansion(mult,add))
    built <- ggplot2::ggplot_build(p)
    panel <- built$layout$panel_params[[1]]
    axis <- panel$x
    keep <- !is.na(axis$breaks)
    cases[[length(cases)+1]] <- list(x=x,explicit=explicit,mult=mult,add=add,
      transformed=built$data[[1]]$x,expanded=panel$x.range,
      breaks=as.list(-axis$breaks[keep]),labels=as.list(axis$get_labels()[keep]),
      projected=built$layout$coord$transform(built$data[[1]],panel)$x)
  }
}
histograms <- lapply(c('right','left'),function(closed) {
 x <- c(0,1,2,3,4,5,6)
 breaks <- c(0,2,4,6)
 p <- ggplot2::ggplot(data.frame(x=x),ggplot2::aes(x)) +
   ggplot2::geom_histogram(breaks=breaks,closed=closed) + ggplot2::scale_x_reverse()
 data <- ggplot2::ggplot_build(p)$data[[1]]
 list(x=x,breaks=breaks,closed=closed,count=data$count,start=data$xmin,end=data$xmax)
})
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases,histograms=histograms),
 'fixtures/parity/ggplot2/reverse-position.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'reverse positional records\n')
