# FIX-GG04: explicit ordinal paint defaults bypass theme palette selection.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else x)
pdf(file=tempfile('ordinal-defaults-',fileext='.pdf'));cases<-list()
for(channel in c('colour','fill'))for(theme_mode in c('absent','supplied'))for(population in c('ordinary','singleton','missing','all_missing','empty')){
 inputs<-switch(population,ordinary=c('c','a','b','a'),singleton='b',missing=c('c',NA,'b','a'),all_missing=NA_character_,empty=character())
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point(shape=21)+do.call(get(paste0('scale_',channel,'_ordinal')),list(guide='none'))
  if(theme_mode=='supplied')p<-p+do.call(theme,setNames(list(function(n)rep('#ff0000',n)),paste0('palette.',channel,'.discrete')))
  b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]][[channel]]),point_count=sum(vapply(pts,function(v)length(v$x),integer(1))))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel=channel,theme_mode=theme_mode,population=population,inputs=encode(inputs),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/default-ordinal-theme-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'ordinal theme defaults\n')
