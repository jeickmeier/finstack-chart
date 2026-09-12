# FIX-GG04: width spelling and parsing through actual Date, datetime and duration panels.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
widths<-c('sec','secs','1 sec','0.5 secs','1.5 min','mins','hour','2 hours','day','2 days','week','2 weeks','month','3 months','year','2 years','1e0 day','+1 day','1 day ','1 sec ignored','1 day ignored','','bogus','seconds','minutes','1 quarter','0 day','-1 day','NaN day','Inf day','1  day',' day','1day','1 Day','1 min extra','1.5 hour','1.5 day','1.5 week','1.5 month','1.5 year','0.5 min','0.5 day','0.5 month','1.1 day','1.1 min')
cases<-list()
for(kind in c('datetime','date','duration'))for(width in widths) {
 span<-if(grepl('sec',width))5.5 else if(grepl('min',width))601.5 else if(grepl('hour',width))30000 else if(grepl('week',width))60*86400 else if(grepl('month',width))400*86400 else if(grepl('year',width))1500*86400 else 21*86400
 x<-switch(kind,datetime=as.POSIXct('2024-01-02 03:04:05.125',tz='UTC')+c(0,span),date=as.Date('2024-01-02')+c(0,ceiling(span/86400)),duration=hms::as_hms(c(-span/5,span)))
 result<-tryCatch({
  scale<-switch(kind,datetime=scale_x_datetime(date_breaks=width,date_labels='%Y-%m-%d %H:%M:%S',expand=expansion(0)),date=scale_x_date(date_breaks=width,date_labels='%Y-%m-%d',expand=expansion(0)),duration=scale_x_time(date_breaks=width,expand=expansion(0)))
  p<-ggplot(data.frame(x=x,y=c(0,1)),aes(x,y))+geom_point()+scale
  a<-suppressWarnings(ggplot_build(p))$layout$panel_params[[1]]$x
  keep<-!is.na(a$breaks)
  list(breaks=as.list(as.numeric(a$breaks[keep])),labels=as.list(unname(a$get_labels()[keep])))
 },error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,width=width,limits=as.list(as.numeric(x)),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/time-width-strings.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'time width string panels\n')
