# FIX-GG04: qualitative type lists select the shortest sufficient vector or fall back to hue.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else x)
pdf(file=tempfile('ordinal-types-',fileext='.pdf'));cases<-list()
palettes<-list(pair=c('red','blue'),shortest=list(c('red','green','blue','black','yellow'),c('orange','purple')),ties=list(c('red','blue'),c('green','black')),named=list(c(b='red',a='blue',z='green',d='black',e='yellow')),duplicate=list(c(a='red',a='blue',c='green',d='black',e='yellow')),empty=character(),empty_list=list(),unknown='not-a-colour')
for(channel in c('colour','fill'))for(palette_name in names(palettes))for(population in c('ordinary','singleton','missing','all_missing','empty')){
 inputs<-switch(population,ordinary=c('c','a','e','b','d'),singleton='b',missing=c('c',NA,'b','a'),all_missing=NA_character_,empty=character())
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point(shape=21)+do.call(get(paste0('scale_',channel,'_discrete')),list(type=palettes[[palette_name]],guide='none'))
  b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]][[channel]]),point_count=sum(vapply(pts,function(v)length(v$x),integer(1))))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel=channel,palette_name=palette_name,palette=lapply(if(is.list(palettes[[palette_name]]))palettes[[palette_name]] else list(palettes[[palette_name]]),function(v)list(values=unname(as.list(v)),names=if(is.null(names(v)))NULL else as.list(names(v)))),population=population,inputs=encode(inputs),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/qualitative-type-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'qualitative type draws\n')
