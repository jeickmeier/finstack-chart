# FIX-GG04: public paint constructor count functions, including shortened results.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
pdf(file=tempfile('binned-functions-',fileext='.pdf'));cases<-list()
for(channel in c('colour','fill'))for(mode in c('full','short','empty','null','missing','named'))for(population in c('ordinary','missing','empty'))for(count in c(2,5,12)){
 inputs<-switch(population,ordinary=c(-2,1,4,10,14),missing=c(NA,1,Inf,4,-Inf),empty=numeric());calls<-list()
 palette<-function(n){calls[[length(calls)+1L]]<<-n;t<-seq_len(n)/max(n,1);value<-ifelse(t<.5,'#ff0000','#0000ff');if(mode=='short')value<-head(value,1);if(mode=='empty')value<-character();if(mode=='null')return(NULL);if(mode=='missing'&&length(value)>1)value[2]<-NA;if(mode=='named')names(value)<-rep(c('c','b','a','d'),length.out=length(value));value}
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point(shape=21)+do.call(get(paste0('scale_',channel,'_binned')),list(palette=palette,n.breaks=count,guide='none'))
  b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]][[channel]]),point_count=sum(vapply(pts,function(v)length(v$x),integer(1))),point_colours=encode(unlist(lapply(pts,function(v)if(channel=='fill')v$gp$fill else v$gp$col),use.names=FALSE)))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel=channel,mode=mode,population=population,count=count,inputs=encode(inputs),calls=calls,result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/binned-constructor-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'binned constructor function draws\n')
