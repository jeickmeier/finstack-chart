# FIX-GG04: independent secondary-unit breaks on shared primary projections.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases<-list()
for (family in c('linear','log','reverse','sqrt')) {
 for (limits in list(c(2,17),c(1,100),c(4,4))) for (affine in list(c(1.8,32),c(-2,10),c(.01,-.3))) {
  factor<-affine[[1]]; offset<-affine[[2]]
  secondary<-ggplot2::sec_axis(function(x)x*factor+offset)
  scale<-switch(family,linear=ggplot2::scale_x_continuous,
    log=ggplot2::scale_x_log10,reverse=ggplot2::scale_x_reverse,sqrt=ggplot2::scale_x_sqrt)
  p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y))+
    ggplot2::geom_point()+scale(limits=limits,sec.axis=secondary)
  panel<-suppressWarnings(tryCatch(ggplot2::ggplot_build(p)$layout$panel_params[[1]],error=function(e)list(error=conditionMessage(e))))
  if(!is.null(panel$error)) {
    cases[[length(cases)+1]]<-list(family=family,limits=limits,factor=factor,offset=offset,error=panel$error)
    next
  }
  info<-panel$x.sec$break_info; order<-order(info$major)
  cases[[length(cases)+1]]<-list(family=family,limits=limits,factor=factor,offset=offset,
    range=info$range,values=as.list(info$major_source_user[order]),
    labels=as.list(info$labels[order]),positions=as.list(info$major[order]))
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),
 'fixtures/parity/ggplot2/secondary-affine.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'affine secondary records\n')
