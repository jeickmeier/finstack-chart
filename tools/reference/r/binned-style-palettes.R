# FIX-GG04: count-based binned shape, linetype and Brewer palettes.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(spaced=c(0,1,4,9),constant=c(4,4,4),missing=c(NA,0,4,NA,9),all_missing=c(NA_real_,NA_real_),empty=numeric(),many=0:20)
cases<-list()
for(palette in c('solid','hollow','linetype','brewer'))for(population in names(sets))for(control in c('default','explicit','empty_breaks','null_breaks','count_eight','count_sixteen','left','limits','partial_limits')){
 value<-sets[[population]];channel<-if(palette=='linetype')'linetype'else if(palette=='brewer')'colour'else'shape'
 args<-list();if(palette%in%c('solid','hollow'))args$solid<-palette=='solid'
 if(palette=='brewer'){args$palette<-'Blues';args$direction<-1}
 if(control=='explicit')args$breaks<-c(1,3,5,7,9)
 if(control=='empty_breaks')args$breaks<-numeric()
 if(control=='null_breaks')args['breaks']<-list(NULL)
 if(control%in%c('count_eight','count_sixteen')){args$n.breaks<-if(control=='count_eight')8 else 16;args$nice.breaks<-FALSE}
 if(control=='left')args$right<-FALSE
 if(control=='limits')args$limits<-c(-1,5)
 if(control=='partial_limits')args$limits<-c(NA,5)
 result<-tryCatch(suppressWarnings({
  scale<-do.call(if(channel=='shape')scale_shape_binned else if(channel=='linetype')scale_linetype_binned else scale_colour_fermenter,args)
  mapping<-aes(x=seq_along(value),y=seq_along(value));mapping[[channel]]<-rlang::new_quosure(quote(value))
  layer<-if(channel=='linetype')geom_segment(aes(xend=seq_along(value),yend=seq_along(value)+.5))else geom_point()
  b<-ggplot_build(ggplot(data.frame(value=value),mapping)+layer+scale);s<-b$plot$scales$get_scales(channel)
  guide<-tryCatch(list(breaks=encode(s$get_breaks()),labels=as.list(unname(s$get_labels()))),error=function(e)list(error=conditionMessage(e)))
  draw_error<-tryCatch({ggplot_gtable(b);NULL},error=function(e)conditionMessage(e))
  list(values=encode(b$data[[1]][[channel]]),limits=encode(s$get_limits()),palette=encode(s$palette.cache),guide=guide,draw_error=draw_error)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(palette=palette,population=population,control=control,inputs=encode(value),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/binned-style-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'binned style/count palette reference builds\n')
