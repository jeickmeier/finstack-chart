# FIX-GG04: elapsed-second break selection and explicit local labels across DST.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
inputs <- list(c(-5.2, 5.2), c(0, 0), c(1700000000, 1700000020),
 c(1710050400,1710061200), c(1730606400,1730620800))
for (zone in c('UTC','America/New_York')) for (i in seq_along(inputs)) {
 for (width in if(i<=3)c(.5,7,90) else c(1800,3600,7200)) {
  limits<-as.POSIXct(inputs[[i]],origin='1970-01-01',tz=zone)
  p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y)) +
    ggplot2::geom_point() + ggplot2::scale_x_datetime(limits=limits,
      expand=ggplot2::expansion(0),date_breaks=paste(width,'sec'),
      date_labels='%Y-%m-%d %H:%M:%S %z',timezone=zone)
  b<-ggplot2::ggplot_build(p); a<-b$layout$panel_params[[1]]$x; keep<-!is.na(a$breaks)
  cases[[length(cases)+1]]<-list(zone=zone,limits=as.numeric(limits),seconds=width,
    breaks=as.list(a$breaks[keep]),labels=as.list(a$get_labels()[keep]))
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),
 'fixtures/parity/ggplot2/time-seconds.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'fixed time records\n')
