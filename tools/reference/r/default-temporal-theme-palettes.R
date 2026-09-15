# FIX-GG04: temporal paint constructors are explicit; temporal numeric defaults are fallbacks.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else x)
pdf(file=tempfile('temporal-theme-',fileext='.pdf'));cases<-list()
for(kind in c('date','datetime'))for(channel in c('colour','fill','size','alpha','linewidth'))for(theme_mode in c('absent','supplied'))for(population in c('ordinary','missing','empty')){
 inputs<-switch(population,ordinary=c(-2,1,4,10,14),missing=c(NA,1,4,NA),empty=numeric());v<-if(kind=='date')as.Date(inputs,origin='1970-01-01')else as.POSIXct(inputs,origin='1970-01-01',tz='UTC');calls<-list()
 palette<-function(x){calls[[length(calls)+1L]]<<-encode(x);if(channel%in%c('colour','fill'))rep('#ff0000',length(x))else if(channel=='alpha')rep(.3,length(x))else rep(3,length(x))}
 result<-tryCatch(suppressWarnings({m<-aes(x,1);m[[channel]]<-quote(v);if(channel=='linewidth'){m$xend<-quote(x+.5);m$yend<-1};p<-ggplot(data.frame(x=seq_along(v),v=v),m)+(if(channel=='linewidth')geom_segment()else if(channel=='fill')geom_point(shape=21)else geom_point());if(theme_mode=='supplied')p<-p+do.call(theme,setNames(list(palette),paste0('palette.',channel,'.continuous')));b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g);panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points')||inherits(v,'segments'),panel$children);list(mapped=encode(b$data[[1]][[channel]]),mark_colours=encode(unlist(lapply(pts,function(v)v$gp$col),use.names=FALSE)))}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(kind=kind,channel=channel,theme_mode=theme_mode,population=population,inputs=encode(inputs),calls=calls,result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/default-temporal-theme-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'temporal theme palette draws\n')
