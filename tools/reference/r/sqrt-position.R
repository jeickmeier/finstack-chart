# FIX-GG04: square-root source-domain clipping versus transformed panel expansion.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
for (x in list(c(0,25,100),c(100,25,0),c(0,0,0),c(4,4,4),c(-1,0,4))) {
 for (explicit in c(FALSE,TRUE)) for (custom in c(FALSE,TRUE)) {
  mult <- if(custom)c(.1,.2)else c(.05,.05)
  add <- if(custom)c(.3,.4)else c(0,0)
  limits <- range(x[x>=0])
  p<-ggplot2::ggplot(data.frame(x=x,y=seq_along(x)),ggplot2::aes(x,y)) +
    ggplot2::geom_point() + ggplot2::scale_x_sqrt(limits=if(explicit)limits else NULL,
      expand=ggplot2::expansion(mult,add))
  b<-suppressWarnings(ggplot2::ggplot_build(p)); panel<-b$layout$panel_params[[1]];a<-panel$x
  keep<-!is.na(a$breaks)
  cases[[length(cases)+1]]<-list(x=x,explicit=explicit,limits=limits,mult=mult,add=add,
    transformed=as.list(b$data[[1]]$x),expanded=panel$x.range,
    breaks=as.list(a$breaks[keep]^2),labels=as.list(a$get_labels()[keep]),
    projected=as.list(b$layout$coord$transform(b$data[[1]],panel)$x))
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),
 'fixtures/parity/ggplot2/sqrt-position.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,na='null')
cat('PASS',length(cases),'sqrt positional records\n')
