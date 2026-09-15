# FIX-GG04: ordinal type vectors use an inclusive Lab ramp at the category count.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else x)
pdf(file=tempfile('ordinal-types-',fileext='.pdf'));cases<-list()
palettes<-list(pair=c('red','blue'),three=c('#440154','#21908c','#fde725'),alpha=c('#ff000040','#0000ffcc'),transparent=c('transparent','red'),single='red',empty=character(),unknown='not-a-colour')
for(channel in c('colour','fill'))for(palette_name in names(palettes))for(population in c('ordinary','singleton','missing','all_missing','empty')){
 inputs<-switch(population,ordinary=c('c','a','e','b','d'),singleton='b',missing=c('c',NA,'b','a'),all_missing=NA_character_,empty=character())
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point(shape=21)+do.call(get(paste0('scale_',channel,'_ordinal')),list(type=palettes[[palette_name]],guide='none'))
  b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]][[channel]]),point_count=sum(vapply(pts,function(v)length(v$x),integer(1))))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel=channel,palette_name=palette_name,palette=as.list(palettes[[palette_name]]),population=population,inputs=encode(inputs),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/ordinal-type-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'ordinal type draws\n')
