# FIX-GG04: positional missing values are replaced after transformation and OOB.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(mixed=c(NA_real_,-9,-1,0,1,3,9),constant=c(4,NA_real_,4),all_missing=c(NA_real_,NA_real_),empty=numeric())
cases<-list()
for(context in c('points','summary'))for(transform in 'duration')for(population in names(sets))for(replacement in c('missing','zero','five','negative'))for(limits in c('auto','fixed'))for(oob in c('censor','squish','keep')){
 inputs<-sets[[population]];seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-encode(x);x}
 na_value<-switch(replacement,missing=NA_real_,zero=0,five=5,negative=-1)
 result<-tryCatch(suppressWarnings({
  args<-list(na.value=na_value,oob=switch(oob,censor=scales::oob_censor,squish=scales::oob_squish,keep=scales::oob_keep))
  if(limits=='fixed')args$limits<-c(1,4)
  if(limits=='function')args$limits<-fun
  if(context=='summary'){
   p<-ggplot(data.frame(x=rep(1,length(inputs)),y=hms::as_hms(inputs)),aes(x,y))+stat_summary(fun=mean,geom='point')+do.call(scale_y_time,args);dimension<-'y'
  }else{
   p<-ggplot(data.frame(x=hms::as_hms(inputs),y=rep(1,length(inputs))),aes(x,y))+geom_point()+do.call(scale_x_time,args);dimension<-'x'
  }
  b<-ggplot_build(p);scale<-if(dimension=='x')b$layout$panel_scales_x[[1]]else b$layout$panel_scales_y[[1]]
  panel<-b$layout$panel_params[[1]][[dimension]]
  list(values=encode(b$data[[1]][[dimension]]),point_positions=tryCatch(encode(panel$rescale(b$data[[1]][[dimension]])),error=function(e)list(error=conditionMessage(e))),limits=encode(scale$get_limits()),range=encode(b$layout$panel_params[[1]][[paste0(dimension,'.range')]]),breaks=encode(panel$breaks),positions=tryCatch(encode(panel$break_positions()),error=function(e)list(error=conditionMessage(e))),labels=as.list(unname(panel$get_labels())))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(context=context,transform=transform,population=population,replacement=replacement,limit_control=limits,oob=oob,inputs=encode(inputs),seen=seen,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-duration-missing.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'duration missing builds\n')
