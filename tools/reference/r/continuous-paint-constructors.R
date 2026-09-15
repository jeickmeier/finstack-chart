# FIX-GG04: public continuous/binned paint constructor defaults and forwarding.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))as.character(x) else x)
pdf(file=tempfile('continuous-constructors-',fileext='.pdf'));cases<-list()
for(channel in c('colour','fill'))for(constructor in c('gradient','gradient2','gradientn','distiller','viridis_c','steps','steps2','stepsn','fermenter','viridis_b'))for(configuration in c('default','custom'))for(population in c('ordinary','singleton','missing','all_missing','empty')){
 inputs<-switch(population,ordinary=c(-4,-1,0,2,6),singleton=1,missing=c(-2,NA,2),all_missing=NA_real_,empty=numeric())
 args<-list();custom<-configuration=='custom'
 if(constructor %in% c('gradient','steps') && custom)args<-list(low='red',high='blue')
 if(constructor %in% c('gradient2','steps2') && custom)args<-list(low='red',mid='white',high='blue',midpoint=1)
 if(constructor %in% c('gradientn','stepsn'))args<-if(custom)list(colors=c('#ff000040','#ffffff80','#0000ffcc'),values=c(0,.2,1))else list(colours=c('red','white','blue'))
 if(constructor %in% c('distiller','fermenter') && custom)args<-c(list(type='div',palette='RdBu',direction=1),if(constructor=='distiller')list(values=c(0,.05,.1,.4,.7,.9,1))else list())
 if(constructor %in% c('viridis_c','viridis_b') && custom)args<-list(alpha=.5,begin=.2,end=.8,direction=-1,option='A',values=c(0,.02,.1,.4,.8,1))
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
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/continuous-paint-constructors.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'continuous/binned constructor draws\n')
