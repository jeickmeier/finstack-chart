# FIX-GG04: automatic datetime defaults after expansion, including DST.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases<-list()
policies<-list(list(mult=c(.05,.05),add=c(0,0)),
              list(mult=c(.1,.2),add=c(.001,.003)),
              list(mult=c(-.2,-.3),add=c(0,0)))
for(zone in c('UTC','America/New_York')) {
 inputs<-if(zone=='UTC')list(c(0,0),c(0,1),c(0,500),c(0,1000),c(0,10000),
  c(1700000000000,1700000000000),c(1700000000000,1700000000001),c(1700000000000,1700000010000)) else
  list(c(1710050400000,1710061200000),c(1730606400000,1730620800000))
 for(limits_ms in inputs) for(policy in policies) {
  limits<-as.POSIXct(limits_ms/1000,origin='1970-01-01',tz=zone)
  p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y))+ggplot2::geom_point()+
   ggplot2::scale_x_datetime(limits=limits,timezone=zone,expand=ggplot2::expansion(mult=policy$mult,add=policy$add))
  a<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x;keep<-!is.na(a$breaks)
  cases[[length(cases)+1]]<-list(zone=zone,limits_ms=limits_ms,policy=policy,
    breaks=as.list(unname(a$breaks[keep])),labels=as.list(a$get_labels()[keep]))
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),
 'fixtures/parity/ggplot2/time-pretty-expansion.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'expanded automatic datetime records\n')
