# FIX-GG04: explicit palette, theme lookup and constructor fallback precedence.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
pdf(file=tempfile('palette-selection-',fileext='.pdf'))
cases<-list()
draw_case<-function(family,channel,source,theme_mode,population,lookup_variant=NULL){
 aesthetics<-if(is.null(lookup_variant))channel else switch(lookup_variant,size_first=c('size','alpha'),alpha_first=c('alpha','size'),missing_first=c('other','size'),alias_aesthetic='color',channel)
 theme_modes<-if(is.null(lookup_variant))setNames(list('first'),paste0('palette.',channel,'.',if(family=='discrete')'discrete'else'continuous'))else{
  modes<-switch(lookup_variant,size_first=c(size='first',alpha='full'),alpha_first=c(size='first',alpha='full'),missing_first=c(size='first',alpha='full'),alias_only=c(color='first'),alias_both=c(colour='index',color='first'),alias_aesthetic=c(colour='first'))
  setNames(as.list(modes),paste0('palette.',names(modes),'.',if(family=='discrete')'discrete'else'continuous'))
 }

 inputs<-if(family=='discrete')switch(population,ordinary=c('c','a','b','a'),missing=c(NA,'b','a',NA),empty=character())else switch(population,ordinary=c(-2,1,4,10,14),missing=c(NA,1,Inf,4,-Inf),empty=numeric())
 calls<-list()
 make_palette<-function(mode){force(mode);function(x){
  calls[[length(calls)+1L]]<<-list(mode=mode,values=encode(x))
  samples<-if(family=='discrete')seq_len(x)/max(x,1)else x
  t<-switch(mode,full=samples,index=seq_along(samples)/max(length(samples),1),first=rep(samples[1],length(samples)))
  if(channel=='colour')ifelse(is.na(t),NA_character_,ifelse(t<.5,'#ff0000','#0000ff'))else if(channel=='size')1+4*t else t
 }}
 result<-tryCatch(suppressWarnings({
  arguments<-list(aesthetics=aesthetics,palette=if(source=='explicit')make_palette('full')else NULL,guide='none')
  if(source=='fallback')arguments$fallback.palette<-make_palette('index')
  if(source=='invalid_fallback')arguments$fallback.palette<-17
  scale<-do.call(switch(family,continuous=continuous_scale,binned=binned_scale,discrete=discrete_scale),arguments)
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  plot<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point()+scale
  if(theme_mode=='supplied'){
   theme_args<-lapply(theme_modes,make_palette)
   plot<-plot+do.call(theme,theme_args)
  }
  built<-ggplot_build(plot);grob<-ggplotGrob(built);grid::grid.draw(grob)
  panel<-grob$grobs[[which(grob$layout$name=='panel')]];points<-Filter(function(g)inherits(g,'points'),panel$children)
  list(mapped=encode(built$data[[1]][[channel]]),point_count=sum(vapply(points,function(g)length(g$x),integer(1))),point_colours=encode(unlist(lapply(points,function(g)g$gp$col),use.names=FALSE)))
 }),error=function(e)list(error=conditionMessage(e)))
 case<-list(family=family,channel=channel,source=source,theme_mode=theme_mode,population=population,inputs=encode(inputs),calls=calls,result=result)
 if(!is.null(lookup_variant)){case$lookup_variant<-lookup_variant;case$lookup_aesthetics<-as.list(aesthetics);case$theme_palettes<-theme_modes}
 case
}
for(family in c('continuous','binned','discrete'))for(channel in c('colour','size','alpha'))for(source in c('explicit','fallback','builtin','invalid_fallback'))for(theme_mode in c('absent','supplied'))for(population in c('ordinary','missing','empty')){
 cases[[length(cases)+1L]]<-draw_case(family,channel,source,theme_mode,population)
}
for(family in c('continuous','binned','discrete'))for(variant in c('size_first','alpha_first','missing_first','alias_only','alias_both','alias_aesthetic'))for(theme_mode in c('absent','supplied'))for(population in c('ordinary','missing','empty')){
 channel<-if(grepl('^alias',variant))'colour'else'size'
 cases[[length(cases)+1L]]<-draw_case(family,channel,'fallback',theme_mode,population,variant)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/scale-palette-selection.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'palette selection draws\n')
