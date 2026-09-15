# FIX-GG04: default shape/linetype theme selection and explicit shape palette bypass.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else x)
pdf(file=tempfile('default-style-',fileext='.pdf'));cases<-list()
for(channel in c('shape','linetype'))for(route in if(channel=='shape')c('automatic','constructor','solid','hollow')else c('automatic','constructor'))for(theme_mode in c('absent','supplied'))for(population in c('ordinary','missing','empty')){
 inputs<-switch(population,ordinary=c('c','a','b','a'),missing=c(NA,'b','a',NA),empty=character());calls<-list()
 palette<-function(n){calls[[length(calls)+1L]]<<-n;if(channel=='shape')rep(3,n)else rep('22',n)}
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x,1);mapping[[channel]]<-quote(v)
  if(channel=='linetype'){mapping$xend<-quote(x+.5);mapping$yend<-1}
  p<-ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+(if(channel=='shape')geom_point()else geom_segment())
  if(route!='automatic')p<-p+do.call(get(paste0('scale_',channel,'_discrete')),if(route%in%c('solid','hollow'))list(solid=route=='solid')else list())
  if(theme_mode=='supplied')p<-p+do.call(theme,setNames(list(palette),paste0('palette.',channel,'.discrete')))
  b<-ggplot_build(p);grid::grid.draw(ggplotGrob(b));list(mapped=encode(b$data[[1]][[channel]]))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(channel=channel,route=route,theme_mode=theme_mode,population=population,inputs=encode(inputs),calls=as.list(calls),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/default-style-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'default style palette draws\n')
