# FIX-GG04: public discrete paint constructor defaults and argument forwarding.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else x)
pdf(file=tempfile('discrete-constructors-',fileext='.pdf'));cases<-list()
for(channel in c('colour','fill'))for(constructor in c('hue','grey','brewer','viridis_d'))for(configuration in c('default','custom'))for(population in c('ordinary','singleton','missing','all_missing','empty')){
 inputs<-switch(population,ordinary=c('c','a','e','b','d'),singleton='b',missing=c('c',NA,'b','a'),all_missing=NA_character_,empty=character())
 args<-if(configuration=='default')list()else switch(constructor,hue=list(h=c(30,300),c=50,l=80,h.start=10,direction=-1),grey=list(start=.1,end=.9),brewer=list(type='div',palette='RdBu',direction=-1),viridis_d=list(alpha=.5,begin=.2,end=.8,direction=-1,option='A'))
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
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/discrete-paint-constructors.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'discrete constructor draws\n')
