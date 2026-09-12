# FIX-GG04: missing aesthetic paint affects drawing after position-scale training.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
populations<-list(mixed=c('a','b',NA,'NA'),missing=rep(NA_character_,4),alternating=c('a',NA,'a',NA))
limits_list<-list(auto=NULL,first=c(NA,'b','a'),only=NA_character_);cases<-list()
for(x_kind in c('numeric','category'))for(y_kind in c('numeric','category'))for(population in names(populations))for(limit_name in names(limits_list))for(translate in c(FALSE,TRUE)){
 x<-if(x_kind=='numeric')0:3 else c('d','c','b','a');y<-if(y_kind=='numeric')4:1 else c('w','z','y','x');v<-populations[[population]]
 b<-ggplot_build(ggplot(data.frame(x=x,y=y,v=v),aes(x,y,colour=v))+geom_point()+scale_colour_hue(limits=limits_list[[limit_name]],na.translate=translate,guide='none'))
 panel<-b$layout$panel_params[[1]];mapped<-b$layout$coord$transform(b$data[[1]],panel)
 cases[[length(cases)+1]]<-list(x_kind=x_kind,y_kind=y_kind,population=population,x=as.list(x),y=as.list(y),values=as.list(v),limits_name=limit_name,limits=if(is.null(limits_list[[limit_name]]))NULL else as.list(limits_list[[limit_name]]),na_translate=translate,result=list(x_positions=as.list(unname(mapped$x)),y_positions=as.list(unname(mapped$y)),x_labels=as.list(unname(panel$x$get_labels())),y_labels=as.list(unname(panel$y$get_labels())),x_major_positions=as.list(unname(panel$x$break_positions())),y_major_positions=as.list(unname(panel$y$break_positions())),x_range=as.list(unname(panel$x$continuous_range)),y_range=as.list(unname(panel$y$continuous_range)),colors=as.list(unname(b$data[[1]]$colour))))
}
jsonlite::write_json(list(cases=cases),'fixtures/parity/ggplot2/missing-paint-positions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'missing paint position panels\n')
