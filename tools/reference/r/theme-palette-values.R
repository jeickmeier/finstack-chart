# FIX-GG04: built-in theme color vectors and their family-specific coercion.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
palettes<-list(pair=c('red','blue'),three=c('red','green','blue'),missing=c('red',NA,'blue'),transparent=c('red','transparent','blue'),alpha=c('#FF000040','#00FF0080','#0000FFFF'),named=c(first='red',last='blue'),empty=character(),singleton='red',numeric=c(.1,.5,.9),invalid=c('red','not-a-color'))
pdf(file=tempfile('theme-colors-',fileext='.pdf'));cases<-list()
for(family in c('continuous','binned','discrete'))for(palette in names(palettes))for(population in c('ordinary','missing','empty'))for(na_mode in c('NA','grey50')){
 inputs<-if(family=='discrete')switch(population,ordinary=c('c','a','d','b'),missing=c(NA,'b','a',NA),empty=character())else switch(population,ordinary=c(-2,1,4,10,14),missing=c(NA,1,Inf,4,-Inf),empty=numeric())
 result<-tryCatch(suppressWarnings({
  scale<-do.call(switch(family,continuous=continuous_scale,binned=binned_scale,discrete=discrete_scale),list(aesthetics='colour',palette=NULL,guide='none',na.value=if(na_mode=='NA')NA else 'grey50'))
  args<-list(palettes[[palette]]);names(args)<-paste0('palette.colour.',if(family=='discrete')'discrete'else'continuous')
  built<-ggplot_build(ggplot(data.frame(x=seq_along(inputs),v=inputs),aes(x,1,colour=v))+geom_point()+scale+do.call(theme,args))
  grob<-ggplotGrob(built);grid::grid.draw(grob);panel<-grob$grobs[[which(grob$layout$name=='panel')]];points<-Filter(function(g)inherits(g,'points'),panel$children)
  list(mapped=encode(built$data[[1]]$colour),point_count=sum(vapply(points,function(g)length(g$x),integer(1))),point_colours=encode(unlist(lapply(points,function(g)g$gp$col),use.names=FALSE)))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(family=family,palette=palette,population=population,na_mode=na_mode,inputs=encode(inputs),palette_values=encode(palettes[[palette]]),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/theme-palette-values.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'built-in theme color vector draws\n')
