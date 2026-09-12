# FIX-GG04: time/date population limits act before summary statistics.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
cases<-list()
for(kind in c('datetime','date')) for(policy in c('censor','squish','keep','coordinate')) {
 values<-c(-2,0,.5,4,NA_real_)
 y<-if(kind=='date')as.Date(values,origin='1970-01-01') else as.POSIXct(values,origin='1970-01-01',tz='UTC')
 limits<-if(kind=='date')as.Date(c(0,1),origin='1970-01-01') else as.POSIXct(c(0,1),origin='1970-01-01',tz='UTC')
 fun<-switch(policy,censor=scales::oob_censor,squish=scales::oob_squish,keep=scales::oob_keep,coordinate=scales::oob_keep)
 scale<-if(kind=='date')ggplot2::scale_y_date(limits=if(policy=='coordinate')NULL else limits,oob=fun) else
   ggplot2::scale_y_datetime(limits=if(policy=='coordinate')NULL else limits,oob=fun,timezone='UTC')
 p<-ggplot2::ggplot(data.frame(x=1,y=y),ggplot2::aes(x,y))+ggplot2::stat_summary(fun=mean,geom='point',na.rm=TRUE)+scale
 if(policy=='coordinate')p<-p+ggplot2::coord_cartesian(ylim=limits)
 b<-ggplot2::ggplot_build(p)
 cases[[length(cases)+1]]<-list(kind=kind,policy=policy,input=as.list(values),mean=b$data[[1]]$y[[1]])
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),'fixtures/parity/ggplot2/time-population.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'timestamp population records\n')
