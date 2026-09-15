# FIX-GG04: public continuous/binned paint constructor defaults and forwarding.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))as.character(x) else x)
pdf(file=tempfile('continuous-constructors-',fileext='.pdf'));cases<-list()
for(channel in c('colour','fill'))for(constructor in c('gradientn','stepsn','distiller','viridis_c'))for(configuration in c('zero','one','identical','all_nan','one_present'))for(population in c('ordinary','missing','empty','singleton','all_missing')){
 inputs<-switch(population,ordinary=c(-4,-1,0,2,6),missing=c(-2,NA,2),empty=numeric(),singleton=1,all_missing=NA_real_)
 args<-list(values=switch(configuration,zero=numeric(),one=0,identical=c(.5,.5,.5),all_nan=c(NA_real_,NA_real_),one_present=c(NA,0,NA)))
 if(constructor %in% c('gradientn','stepsn'))args$colours<-c('red','white','blue')
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point(shape=21)+do.call(get(paste0('scale_',channel,'_',constructor)),c(args,list(guide='none')))
  b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]][[channel]]),point_count=sum(vapply(pts,function(v)length(v$x),integer(1))))
 }),error=function(e)list(error=conditionMessage(e)))
 args$values<-lapply(args$values,function(v)if(is.na(v))list(number='NaN')else if(is.infinite(v))list(number=if(v>0)'Infinity'else '-Infinity')else v)
 cases[[length(cases)+1L]]<-list(channel=channel,constructor=constructor,configuration=configuration,args=args,population=population,inputs=encode(inputs),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/gradient-invalid-constructors.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'invalid gradient remapping constructor draws\n')
