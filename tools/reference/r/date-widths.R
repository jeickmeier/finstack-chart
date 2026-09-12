# FIX-GG04: Date width controls retain day-based formatting.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
cases<-list()
for(limits_days in list(c(-5,5),c(.25,17.75),c(19000,19200))) for(width in c('2 days','2 weeks','2 months','2 years')) {
 limits<-as.Date(limits_days,origin='1970-01-01')
 p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y))+ggplot2::geom_point()+
  ggplot2::scale_x_date(limits=limits,expand=ggplot2::expansion(0),date_breaks=width)
 a<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x;keep<-!is.na(a$breaks)
 cases[[length(cases)+1]]<-list(limits_days=limits_days,width=width,breaks=as.list(unname(a$breaks[keep])),labels=as.list(a$get_labels()[keep]))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),'fixtures/parity/ggplot2/date-widths.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'Date width records\n')
