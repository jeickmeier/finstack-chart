# FIX-GG04: uncropped temporal aesthetic widths and the shared reference string grammar.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
widths<-c('2 secs','2 mins','2 hours','2 days','2 weeks','2 months','2 years','1.5 secs','1.5 mins','1.5 days','2 secs extra','2 mins extra','2 months extra','0 secs','bogus')
cases<-list()
for(kind in c('date','datetime'))for(width in widths)for(population in c('boundary','offset')){
 multiplier<-if(grepl('year',width))365*86400 else if(grepl('month',width))31*86400 else if(grepl('week',width))7*86400 else if(grepl('day',width))86400 else if(grepl('hour',width))3600 else if(grepl('min',width))60 else 1
 if(kind=='date')multiplier<-max(1,multiplier/86400)
 inputs<-c(0,1,4,9)*multiplier+if(population=='offset')multiplier/4 else 0
 when<-if(kind=='date')as.Date(inputs+19723,origin='1970-01-01')else as.POSIXct(inputs+1704067200,origin='1970-01-01',tz='UTC')
 result<-tryCatch(suppressWarnings({
  s<-do.call(if(kind=='date')scale_size_date else scale_size_datetime,list(date_breaks=width))
  b<-ggplot_build(ggplot(data.frame(when=when),aes(x=seq_along(when),y=seq_along(when),size=when))+geom_point()+s)
  s<-b$plot$scales$get_scales('size')
  guide<-list(breaks=encode(s$get_breaks()),labels=as.list(unname(s$get_labels())))
  ggplot_gtable(b)
  list(values=encode(b$data[[1]]$size),limits=encode(s$get_limits()),guide=guide)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,population=population,control='width',width=width,inputs=encode(as.numeric(when)),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/temporal-aesthetic-widths.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'temporal aesthetic width reference builds\n')
