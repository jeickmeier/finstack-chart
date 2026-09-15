# FIX-GG04: public continuous/binned paint constructor defaults and forwarding.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))as.character(x) else x)
pdf(file=tempfile('continuous-constructors-',fileext='.pdf'));cases<-list()
for(channel in c('colour','fill'))for(constructor in c('gradientn','stepsn','distiller','viridis_c'))for(configuration in c('short','long','duplicate','descending'))for(population in c('ordinary','missing','empty')){
 inputs<-switch(population,ordinary=c(-4,-1,0,2,6),missing=c(-2,NA,2),empty=numeric())
 args<-list(values=switch(configuration,short=c(0,1),long=c(0,.1,.3,.7,.9,1,1.2,1.4),duplicate=c(0,.3,.3,1),descending=c(1,.8,.2,0)))
 if(constructor %in% c('gradientn','stepsn'))args$colours<-c('red','white','blue')
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point(shape=21)+do.call(get(paste0('scale_',channel,'_',constructor)),c(args,list(guide='none')))
  b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]][[channel]]),point_count=sum(vapply(pts,function(v)length(v$x),integer(1))))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel=channel,constructor=constructor,configuration=configuration,args=args,population=population,inputs=encode(inputs),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/gradient-remap-constructors.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'gradient remapping constructor draws\n')
