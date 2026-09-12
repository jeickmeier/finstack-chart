# FIX-GG04: Date expansion, automatic breaks and labels use day units.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases<-list()
for(limits_days in list(c(0,0),c(0,1),c(.25,2.75),c(-5,5),c(10,30),c(0,100),c(0,400),c(19000,19100),c(19000,22000))) {
 for(policy in list(list(mult=c(0,0),add=c(0,0)),list(mult=c(.05,.05),add=c(0,0)),list(mult=c(.1,.2),add=c(.25,.75)))) {
  for(n in c(3,5,8)) {
   limits<-as.Date(limits_days,origin='1970-01-01')
   p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y))+ggplot2::geom_point()+
    ggplot2::scale_x_date(limits=limits,expand=ggplot2::expansion(mult=policy$mult,add=policy$add),breaks=scales::breaks_pretty(n))
   a<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x;keep<-!is.na(a$breaks)
   cases[[length(cases)+1]]<-list(limits_days=limits_days,policy=policy,n=n,
    range=as.list(a$continuous_range),breaks=as.list(unname(a$breaks[keep])),labels=as.list(a$get_labels()[keep]),
    position=as.list(a$rescale(limits_days)))
  }
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),
 'fixtures/parity/ggplot2/date-scales.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'Date records\n')
