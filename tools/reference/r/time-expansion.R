# FIX-GG04: datetime expansion is measured in seconds, including constant domains.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
inputs <- list(c(-5200,5200),c(0,0),c(0,1),c(1700000000000,1700000020000))
policies <- list(list(mult=c(.05,.05),add=c(0,0)),
                 list(mult=c(0,0),add=c(0,0)),
                 list(mult=c(.1,.2),add=c(.001,.003)),
                 list(mult=c(-.2,-.3),add=c(0,0)))
for (limits_ms in inputs) for (policy in policies) {
 limits<-as.POSIXct(limits_ms/1000,origin='1970-01-01',tz='UTC')
 p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y)) +
  ggplot2::geom_point() + ggplot2::scale_x_datetime(limits=limits,
   expand=ggplot2::expansion(mult=policy$mult,add=policy$add),date_breaks='0.001 sec',
   date_labels='%S',timezone='UTC')
 # Inspect range without enumerating millisecond breaks for a wide panel.
 p$scales$scales[[1]]$breaks <- NULL
 b<-ggplot2::ggplot_build(p); a<-b$layout$panel_params[[1]]$x
 numeric_p<-ggplot2::ggplot(data.frame(x=as.numeric(limits),y=c(0,1)),ggplot2::aes(x,y)) +
  ggplot2::geom_point() + ggplot2::scale_x_continuous(limits=as.numeric(limits),
   expand=ggplot2::expansion(mult=policy$mult,add=policy$add),breaks=NULL)
 numeric_axis<-ggplot2::ggplot_build(numeric_p)$layout$panel_params[[1]]$x
 cases[[length(cases)+1]]<-list(numeric_range=as.list(numeric_axis$continuous_range),
  numeric_position=as.list(numeric_axis$rescale(as.numeric(limits))),limits_ms=limits_ms,policy=policy,
  range=as.list(a$continuous_range),relative_range=as.list(a$continuous_range-as.numeric(limits[1])),position=as.list(a$rescale(as.numeric(limits))))
}
fine_cases <- list()
for (ns in c('1700000000000001000','1700000000000010000','1700000000000100000')) {
 raw<-c('1700000000000000000',ns)
 limits<-as.POSIXct(as.numeric(raw)/1e9,origin='1970-01-01',tz='UTC')
 p<-ggplot2::ggplot(data.frame(x=limits,y=c(0,1)),ggplot2::aes(x,y)) +
  ggplot2::geom_point() + ggplot2::scale_x_datetime(limits=limits,breaks=NULL)
 a<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x
 fine_cases[[length(fine_cases)+1]]<-list(limits_ns=raw,range=as.list(a$continuous_range),relative_range=as.list(a$continuous_range-1700000000))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases,fine_cases=fine_cases),
 'fixtures/parity/ggplot2/time-expansion.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'datetime expansion records\n')
