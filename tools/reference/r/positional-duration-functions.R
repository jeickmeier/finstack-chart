# FIX-GG04: actual duration positional callbacks before and after statistics.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(as.numeric(v)),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(spaced=c(0,1,4,9),constant=c(4,4),missing=c(NA,0,9),all_missing=c(NA_real_,NA_real_),empty=numeric(),fractional=c(0.125,1.5,4.75))
cases<-list()
for(kind in 'duration')for(context in c('points','summary'))for(population in names(sets))for(control in c('identity','reverse','fixed','lower_zero','missing_lower','empty','single'))for(oob in c('censor','squish','keep')){
 convert<-function(v)hms::as_hms(v)
 inputs<-convert(sets[[population]]);seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-list(class=as.list(class(x)),values=encode(x));switch(control,identity=x,reverse=rev(x),fixed=convert(c(0,10)),lower_zero=c(convert(0),x[2]),missing_lower=c(convert(NA_real_),x[2]),empty=convert(numeric()),single=convert(5))}
 result<-tryCatch(suppressWarnings({
  scale<-if(context=='points')scale_x_time else scale_y_time
  args<-list(limits=fun,oob=switch(oob,censor=scales::oob_censor,squish=scales::oob_squish,keep=scales::oob_keep))
  if(context=='summary'){
   p<-ggplot(data.frame(x=rep(1,length(inputs)),y=inputs),aes(x,y))+stat_summary(fun=mean,geom='point')+do.call(scale,args);dimension<-'y'
  }else{
   p<-ggplot(data.frame(x=inputs,y=rep(1,length(inputs))),aes(x,y))+geom_point()+do.call(scale,args);dimension<-'x'
  }
  b<-ggplot_build(p);s<-if(dimension=='x')b$layout$panel_scales_x[[1]]else b$layout$panel_scales_y[[1]];panel<-b$layout$panel_params[[1]][[dimension]]
  list(values=encode(b$data[[1]][[dimension]]),point_positions=tryCatch(encode(panel$rescale(b$data[[1]][[dimension]])),error=function(e)list(error=conditionMessage(e))),limits=encode(s$get_limits()),range=encode(b$layout$panel_params[[1]][[paste0(dimension,'.range')]]),breaks=encode(panel$breaks),positions=tryCatch(encode(panel$break_positions()),error=function(e)list(error=conditionMessage(e))),labels=as.list(unname(panel$get_labels())))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,context=context,population=population,control=control,oob=oob,inputs=encode(inputs),seen=seen,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-duration-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'positional duration callback builds\n')
