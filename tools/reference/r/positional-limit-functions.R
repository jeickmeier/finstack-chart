# FIX-GG04: positional limit functions participate before and after statistics.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(spaced=c(1,3,9),constant=c(4,4),missing=c(NA,1,9),all_missing=c(NA_real_,NA_real_),empty=numeric(),invalid=c(-9,-1,0,4))
capture_case<-function(kind,transform,population,control,oob=NULL){
 inputs<-sets[[population]];seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-encode(x);switch(control,identity=x,reverse=rev(x),fixed=c(0,10),lower_zero=c(0,x[2]),missing_lower=c(NA_real_,x[2]),empty=numeric(),single=5)}
 result<-tryCatch(suppressWarnings({
  constructor<-if(kind=='continuous')scale_x_continuous else scale_x_binned
  args<-list(limits=fun,transform=transform)
  if(!is.null(oob))args$oob<-switch(oob,censor=scales::oob_censor,squish=scales::oob_squish,keep=scales::oob_keep)
  scale<-do.call(constructor,args)
  b<-ggplot_build(ggplot(data.frame(x=inputs,y=rep(1,length(inputs))),aes(x,y))+geom_point()+scale)
  s<-b$layout$panel_scales_x[[1]];panel<-b$layout$panel_params[[1]]
  list(values=encode(b$data[[1]]$x),coordinate_positions=tryCatch(encode(b$layout$coord$transform(b$data[[1]],panel)$x),error=function(e)list(error=conditionMessage(e))),point_positions=tryCatch(encode(panel$x$rescale(b$data[[1]]$x)),error=function(e)list(error=conditionMessage(e))),limits=encode(s$get_limits()),range=encode(panel$x.range),breaks=encode(panel$x$breaks),positions=tryCatch(encode(panel$x$break_positions()),error=function(e)list(error=conditionMessage(e))),labels=as.list(unname(panel$x$get_labels())))
 }),error=function(e)list(error=conditionMessage(e)))
 record<-list(kind=kind,transform=transform,population=population,control=control,inputs=encode(inputs),seen=seen,result=result)
 if(!is.null(oob))record$oob<-oob
 record
}
cases<-list();argument_cases<-list()
for(kind in c('continuous','binned'))for(transform in c('identity','sqrt','reverse','log10'))for(population in names(sets))for(control in c('identity','reverse','fixed','lower_zero','missing_lower','empty','single')){
 if(transform!='log10')cases[[length(cases)+1]]<-capture_case(kind,transform,population,control)
 for(oob in c('censor','squish','keep')){
  default<-if(kind=='continuous')'censor'else'squish'
  if(transform=='log10'||oob!=default)argument_cases[[length(argument_cases)+1]]<-capture_case(kind,transform,population,control,oob)
 }
}

stage_cases<-list()
for(kind in c('continuous','binned'))for(context in c('summary','shared'))for(transform in c('identity','sqrt','reverse'))for(control in c('identity','lower_zero','fixed','single')){
 seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-encode(x);switch(control,identity=x,lower_zero=c(0,x[2]),fixed=c(0,10),single=5)}
 result<-tryCatch(suppressWarnings({
  scale<-if(kind=='continuous')scale_x_continuous else scale_x_binned
  yscale<-if(kind=='continuous')scale_y_continuous else scale_y_binned
  if(context=='summary'){
   p<-ggplot(data.frame(x=1,y=c(1,3,9)),aes(x,y))+stat_summary(fun=mean,geom='point')+yscale(limits=fun,transform=transform)
  }else{
   p<-ggplot(mapping=aes(x=x,y=1))+geom_point(data=data.frame(x=c(1,3)))+geom_point(data=data.frame(x=9))+scale(limits=fun,transform=transform)
  }
  b<-ggplot_build(p);dimension<-if(context=='summary')'y'else'x'
  scales<-if(context=='summary')b$layout$panel_scales_y else b$layout$panel_scales_x
  list(values=lapply(b$data,function(d)encode(d[[dimension]])),limits=encode(scales[[1]]$get_limits()))
 }),error=function(e)list(error=conditionMessage(e)))
 stage_cases[[length(stage_cases)+1]]<-list(kind=kind,context=context,transform=transform,control=control,seen=seen,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases,stage_cases=stage_cases),'fixtures/parity/ggplot2/positional-limit-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'positional limit-function reference builds\n')

jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=argument_cases),'fixtures/parity/ggplot2/positional-limit-function-arguments.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(argument_cases),'positional callback argument builds\n')
