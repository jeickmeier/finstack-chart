# FIX-GG04: default constructors consult theme palettes; explicit range/area/radius bypass.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
pdf(file=tempfile('default-palette-',fileext='.pdf'));cases<-list()
for(channel in c('colour','fill','size','alpha','linewidth'))for(family in c('continuous','discrete'))for(route in c('automatic','constructor','range'))for(theme_mode in c('absent','supplied')){
 if(route=='range'&&channel%in%c('colour','fill'))next
 inputs<-if(family=='discrete')c('c','a','b','a')else c(-2,1,4,10,14)
 calls<-list();palette<-function(x){calls[[length(calls)+1L]]<<-encode(x);samples<-if(family=='discrete')seq_len(x)/max(x,1)else x;if(channel%in%c('colour','fill'))rep('#ff0000',length(samples))else if(channel=='alpha')rep(.3,length(samples))else rep(3,length(samples))}
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+if(channel=='fill')geom_point(shape=21)else geom_point()
  if(route!='automatic'){
   arguments<-list();if(route=='range')arguments$range<-c(.2,.8)
   p<-p+do.call(get(paste0('scale_',channel,'_',if(family=='discrete')if(channel%in%c('colour','fill'))'discrete'else'ordinal'else'continuous')),arguments)
  }
  if(theme_mode=='supplied')p<-p+do.call(theme,setNames(list(palette),paste0('palette.',channel,'.',family)))
  b<-ggplot_build(p);g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]][[channel]]),point_colours=encode(unlist(lapply(pts,function(v)v$gp$col),use.names=FALSE)))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel=channel,family=family,route=route,theme_mode=theme_mode,inputs=encode(inputs),calls=calls,result=result)
}
for(kind in c('area','radius'))for(theme_mode in c('absent','supplied')){
 inputs<-c(-2,1,4,10,14);calls<-list();palette<-function(x){calls[[length(calls)+1L]]<<-encode(x);rep(3,length(x))}
 result<-tryCatch(suppressWarnings({p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),aes(x,1,size=v))+geom_point()+do.call(get(if(kind=='radius')'scale_radius'else'scale_size_area'),list());if(theme_mode=='supplied')p<-p+theme(palette.size.continuous=palette);b<-ggplot_build(p);grid::grid.draw(ggplotGrob(b));list(mapped=encode(b$data[[1]]$size))}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel='size',family='continuous',route=kind,theme_mode=theme_mode,inputs=encode(inputs),calls=calls,result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/default-palette-selection.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'default palette draws\n')
