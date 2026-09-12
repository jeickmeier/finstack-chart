# FIX-GG04: local anchor plus elapsed or calendar progression.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
inputs <- list(c('2024-03-09 05:17:00','2024-03-13 07:11:00'),
               c('2024-11-02 05:17:00','2024-11-06 07:11:00'),
               c('2024-02-17 05:17:00','2025-03-08 07:11:00'))
for (zone in c('UTC','America/New_York')) for (i in seq_along(inputs)) {
 for (width in if(i<=2)c('7 hours','2 days','2 weeks') else c('2 months','2 years')) {
  limits<-as.POSIXct(inputs[[i]],tz=zone)
  p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y)) +
    ggplot2::geom_point() + ggplot2::scale_x_datetime(limits=limits,
      expand=ggplot2::expansion(0),date_breaks=width,
      date_labels='%Y-%m-%d %H:%M:%S %z',timezone=zone)
  b<-ggplot2::ggplot_build(p); a<-b$layout$panel_params[[1]]$x; keep<-!is.na(a$breaks)
  cases[[length(cases)+1]]<-list(zone=zone,limits=as.numeric(limits),width=width,
    breaks=as.list(a$breaks[keep]),labels=as.list(a$get_labels()[keep]))
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),
 'fixtures/parity/ggplot2/time-widths.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'time width records\n')
