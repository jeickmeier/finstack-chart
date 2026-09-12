# FIX-GG04: Date/datetime guide arguments over a nonpositional numeric palette.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity' else '-Infinity' else x)
sets<-list(short=c(0,1,2,3),spaced=c(0,1,4,9),constant=c(4,4,4),missing=c(NA,0,4,NA,9),all_missing=c(NA_real_,NA_real_),empty=numeric())
controls<-c('default','null_breaks','empty_breaks','explicit_breaks','null_labels','explicit_labels','bad_labels','width','format','count_two','limits','partial_limits','hidden_guide')
cases<-list()
for(kind in c('date','datetime'))for(population in names(sets))for(control in controls){
 convert<-function(v)if(kind=='date')as.Date(v+19723,origin='1970-01-01')else as.POSIXct(v+1704067200,origin='1970-01-01',tz='UTC')
 when<-convert(sets[[population]]);args<-list()
 if(control=='null_breaks')args['breaks']<-list(NULL)
 if(control=='empty_breaks')args$breaks<-convert(numeric())
 if(control=='explicit_breaks')args$breaks<-convert(c(-1,0,2,5))
 if(control=='null_labels')args['labels']<-list(NULL)
 if(control%in%c('explicit_labels','bad_labels')){args$breaks<-convert(c(-1,0,2,5));args$labels<-if(control=='bad_labels')c('A','B')else c('Before','Start','Two','After')}
 if(control=='width')args$date_breaks<-if(kind=='date')'2 days'else'2 secs'
 if(control=='format')args$date_labels<-if(kind=='date')'%Y-%m-%d'else'%H:%M:%S'
 if(control=='count_two')args$n.breaks<-2
 if(control=='limits')args$limits<-convert(c(-1,5))
 if(control=='partial_limits')args$limits<-convert(c(NA,5))
 if(control=='hidden_guide')args$guide<-'none'
 result<-tryCatch(suppressWarnings({
  s<-do.call(if(kind=='date')scale_size_date else scale_size_datetime,args)
  p<-ggplot(data.frame(when=when),aes(x=seq_along(when),y=seq_along(when),size=when))+geom_point()+s
  b<-ggplot_build(p);s<-b$plot$scales$get_scales('size')
  guide<-tryCatch(list(breaks=encode(s$get_breaks()),labels=if(is.null(s$get_labels()))NULL else as.list(unname(s$get_labels()))),error=function(e)list(error=conditionMessage(e)))
  draw_error<-tryCatch({ggplot_gtable(b);NULL},error=function(e)conditionMessage(e))
  list(values=encode(b$data[[1]]$size),limits=encode(s$get_limits()),guide=guide,draw_error=draw_error)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,population=population,control=control,inputs=encode(as.numeric(when)),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/temporal-aesthetic-guides.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'temporal aesthetic guide argument builds\n')
