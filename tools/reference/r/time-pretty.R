# FIX-GG04: automatic datetime breaks and labels, pinned independently of core.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
for (zone in c('UTC','America/New_York')) {
 for (start in if(zone=='UTC') c(-1000000000,0,1700000000) else c(1710050400,1730606400)) {
  for (span in c(0,.001,.5,1,2,5,10,30,90,600,3600,36000,100000,1000000,10000000,if(zone=='UTC')c(100000000,1000000000,10000000000))) {
   for (n in c(1,2.5,3,5,8,16)) {
    limits<-as.POSIXct(c(start,start+span),origin='1970-01-01',tz=zone)
    p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y)) +
     ggplot2::geom_point() + ggplot2::scale_x_datetime(limits=limits,expand=ggplot2::expansion(0),
      breaks=scales::breaks_pretty(n),timezone=zone)
    a<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x; keep<-!is.na(a$breaks)
    cases[[length(cases)+1]]<-list(zone=zone,limits=as.list(as.numeric(limits)),n=n,
     breaks=as.list(unname(a$breaks[keep])),labels=as.list(a$get_labels()[keep]))
   }
  }
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),
 'fixtures/parity/ggplot2/time-pretty.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'automatic datetime records\n')
