# FIX-GG04: explicit timestamp minor candidates retain fractional Date values.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.nan(x))'NaN'else if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else unname(x))
cases<-list()
for(kind in c('date','datetime'))for(window in if(kind=='date')c('short','long','pre_epoch')else c('short','long')){
 if(kind=='date'){
  input<-as.Date('2024-01-01')+if(window=='short')c(0,31)else c(57,423)
  if(window=='pre_epoch')input<-as.Date('1969-12-01')+c(0,31)
 }else if(kind=='datetime'){
  input<-as.POSIXct('2024-03-10 00:00:00',tz='UTC')+if(window=='short')c(0,3*86400)else c(0,180*86400)
 }else{
  input<-hms::hms(seconds=if(window=='short')c(-10.5,15.5)else c(-200000,200000))
 }
 for(major in c('regular','descending','outside','missing','empty')){
  result<-tryCatch(suppressWarnings({
   candidate<-switch(major,outside=input[1]+c(-1,0,1,2)*(as.numeric(input[2])-as.numeric(input[1])),missing=c(input[1],NA,input[2]),regular=input[1]+c(0,.25,.75,1)*(as.numeric(input[2])-as.numeric(input[1])),descending=input[1]+c(1,.75,.25,0)*(as.numeric(input[2])-as.numeric(input[1])),one=input[1],empty=input[FALSE])
   s<-switch(kind,date=scale_x_date(minor_breaks=candidate),datetime=scale_x_datetime(minor_breaks=candidate,timezone='UTC'),duration=scale_x_time(minor_breaks=candidate))
   b<-ggplot_build(ggplot(data.frame(x=input,y=c(1,2)),aes(x,y))+geom_point()+s);p<-b$layout$panel_params[[1]]$x
   position<-p$rescale(p$minor_breaks);keep<-is.finite(p$minor_breaks)&is.finite(position)
   list(values=encode(p$minor_breaks[keep]),positions=encode(position[keep]),range=encode(p$continuous_range))
  }),error=function(e)list(error=conditionMessage(e)))
  cases[[length(cases)+1]]<-list(kind=kind,window=window,minor=major,minor_values=encode(as.numeric(candidate)),inputs=encode(as.numeric(input)),result=result)
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/minor-time-values.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'explicit timestamp minor panels\n')
