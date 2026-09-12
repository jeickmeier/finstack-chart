# FIX-GG04 / GG2-03: absolute-double datetime rescaling versus exact source ticks.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases<-list()
for(epoch in c(0,1704067200,-1704067200))for(span in c(9,.01,.001,.0001,.00001,.000001))for(channel in c('size','alpha','linewidth','colour','fill'))for(hidden in c(FALSE,TRUE)){
 offsets<-c(0,.25,.5,1)*span
 when<-as.POSIXct(epoch+offsets,origin='1970-01-01',tz='UTC')
 mapping<-aes(x=seq_along(when),y=seq_along(when));mapping[[channel]]<-rlang::new_quosure(quote(when))
 p<-ggplot(data.frame(when=when),mapping)+(if(channel=='linewidth')geom_line()else geom_point(shape=21))
 if(hidden)p<-p+guides(!!!setNames(list('none'),channel))
 b<-suppressWarnings(ggplot_build(p));s<-b$plot$scales$get_scales(channel)
 cases[[length(cases)+1]]<-list(epoch=epoch,offsets=as.list(offsets),span=span,channel=channel,hidden=hidden,inputs=as.list(as.numeric(when)),values=as.list(unname(b$data[[1]][[channel]])),zero_range=scales::zero_range(s$get_limits()))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/temporal-aesthetic-precision.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'temporal precision reference builds\n')
