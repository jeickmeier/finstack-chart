# FIX-GG04: user-defined strictly monotone secondary transformations.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases<-list()
for (family in c('linear','log','reverse','sqrt')) {
 for (kind in c('cube','square','piecewise','decreasing')) {
  limits<-if(kind=='cube'&&family%in%c('linear','reverse'))c(-2,3)else c(2,17)
  trans<-switch(kind,cube=function(x)x^3,square=function(x)x^2,
    piecewise=stats::approxfun(c(0,5,20),c(0,2,10),rule=2),
    decreasing=stats::approxfun(c(0,5,20),c(10,2,0),rule=2))
  scale<-switch(family,linear=ggplot2::scale_x_continuous,
    log=ggplot2::scale_x_log10,reverse=ggplot2::scale_x_reverse,sqrt=ggplot2::scale_x_sqrt)
  # Piecewise reference functions remain inside their authored knots; no extrapolation claim.
  p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y))+
    ggplot2::geom_point()+scale(limits=limits,expand=ggplot2::expansion(0),
      sec.axis=ggplot2::sec_axis(trans))
  panel<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]
  info<-panel$x.sec$break_info; order<-order(info$major)
  cases[[length(cases)+1]]<-list(family=family,kind=kind,limits=limits,
    range=info$range,values=as.list(info$major_source_user[order]),
    labels=as.list(info$labels[order]),positions=as.list(info$major[order]))
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),
 'fixtures/parity/ggplot2/secondary-transform.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'transformed secondary records\n')
