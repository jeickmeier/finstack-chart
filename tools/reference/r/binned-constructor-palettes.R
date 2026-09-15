# FIX-GG04: public binned paint constructors coerce explicit palettes to count palettes.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
pdf(file=tempfile('binned-constructor-',fileext='.pdf'));cases<-list()
choices<-list(viridis='viridis',brewer='Set1',hue='hue',hcl='Blue-Red 3',manual=c('#ff0000','#00ff00','#0000ff'),unknown='not-a-palette')
for(channel in c('colour','fill'))for(palette_name in names(choices))for(population in c('ordinary','missing','empty'))for(count in c(2,5,12)){
 inputs<-switch(population,ordinary=c(-2,1,4,10,14),missing=c(NA,1,Inf,4,-Inf),empty=numeric());palette<-choices[[palette_name]]
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point(shape=21)+do.call(get(paste0('scale_',channel,'_binned')),list(palette=palette,n.breaks=count,guide='none'))
  b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]][[channel]]),point_count=sum(vapply(pts,function(v)length(v$x),integer(1))),point_colours=encode(unlist(lapply(pts,function(v)if(channel=='fill')v$gp$fill else v$gp$col),use.names=FALSE)))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel=channel,palette_name=palette_name,palette=encode(palette),population=population,count=count,inputs=encode(inputs),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/binned-constructor-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'binned constructor palette draws\n')
