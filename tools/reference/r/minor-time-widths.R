# FIX-GG04: explicit minor widths share the date/time/elapsed alignment owners.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(v,function(x)if(is.nan(x))'NaN'else if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else unname(x))
cases<-list()
for(kind in c('date','datetime','duration'))for(window in c('short','long')){
 if(kind=='date'){
  input<-as.Date('2024-01-01')+if(window=='short')c(0,31)else c(57,423)
  widths<-c('1 day','2 days','1 week','1 month','2 months','1 year')
 }else if(kind=='datetime'){
  input<-as.POSIXct('2024-03-10 00:00:00',tz='UTC')+if(window=='short')c(0,3*86400)else c(0,180*86400)
  widths<-if(window=='short')c('1 hour','6 hours','1 day','1 week','1 month')else c('6 hours','1 day','1 week','1 month','1 year')
 }else{
  input<-hms::hms(seconds=if(window=='short')c(-10.5,15.5)else c(-200000,200000))
  widths<-if(window=='short')c('0.1 seconds','1 second','5 seconds','1 minute','1 hour')else c('1 hour','6 hours','1 day')
 }
 for(width in widths){
  result<-tryCatch(suppressWarnings({
   s<-switch(kind,date=scale_x_date(date_minor_breaks=width),datetime=scale_x_datetime(date_minor_breaks=width,timezone='UTC'),duration=scale_x_time(date_minor_breaks=width))
   b<-ggplot_build(ggplot(data.frame(x=input,y=c(1,2)),aes(x,y))+geom_point()+s);p<-b$layout$panel_params[[1]]$x
   position<-p$rescale(p$minor_breaks);keep<-is.finite(p$minor_breaks)&is.finite(position)
   list(values=encode(p$minor_breaks[keep]),positions=encode(position[keep]),range=encode(p$continuous_range))
  }),error=function(e)list(error=conditionMessage(e)))
  cases[[length(cases)+1]]<-list(kind=kind,window=window,width=width,inputs=encode(as.numeric(input)),result=result)
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/minor-time-widths.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'minor time-width panels\n')
