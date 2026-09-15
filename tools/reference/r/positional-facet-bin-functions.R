# FIX-GG04: source and post-statistic callbacks train on shared or free facet scales.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(as.numeric(v)),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
sets<-list(balanced=c(1,4,9,16),constant=c(4,4,9,9),missing=c(NA,NA,9,16),empty_panel=c(1,4,9,16))
cases<-list()
for(context in c('points','summary'))for(transform in c('identity','sqrt','reverse'))for(population in names(sets))for(policy in c('fixed','free'))for(control in c('automatic','identity','reverse','fixed','lower_zero','missing_lower','empty','single'))for(oob in c('censor','squish','keep')){
 values<-sets[[population]];levels<-if(population=='empty_panel')c('A','B','C')else c('A','B');groups<-factor(c('A','A','B','B'),levels=levels);seen<-list()
 fun<-function(x){seen[[length(seen)+1]]<<-encode(x);switch(control,identity=x,reverse=rev(x),fixed=c(0,10),lower_zero=c(0,x[2]),missing_lower=c(NA_real_,x[2]),empty=numeric(),single=5)}
 result<-tryCatch(suppressWarnings({
  dimension<-if(context=='summary')'y'else'x';scale<-if(context=='summary')scale_y_binned else scale_x_binned
  args<-list(transform=transform,limits=if(control=='automatic')NULL else fun,oob=switch(oob,censor=scales::oob_censor,squish=scales::oob_squish,keep=scales::oob_keep))
  data<-data.frame(v=values,panel=groups)
  if(context=='summary')p<-ggplot(data,aes(x=1,y=v))+stat_summary(fun=mean,geom='point') else p<-ggplot(data,aes(x=v,y=1))+geom_point()
  p<-p+do.call(scale,args)+facet_wrap(vars(panel),scales=if(policy=='fixed')'fixed'else if(context=='summary')'free_y'else'free_x',drop=FALSE)
  b<-ggplot_build(p);panels<-list()
  for(i in seq_along(b$layout$panel_params)){
   panel<-b$layout$panel_params[[i]][[dimension]];layer<-b$data[[1]];observed<-layer[layer$PANEL==i,dimension]
   panels[[i]]<-list(key=as.character(b$layout$layout$panel[i]),values=encode(observed),point_positions=encode(panel$rescale(observed)),limits=encode(panel$limits),range=encode(b$layout$panel_params[[i]][[paste0(dimension,'.range')]]),breaks=encode(panel$breaks),positions=encode(panel$break_positions()),labels=as.list(unname(panel$get_labels())))
  }
  list(panels=panels)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(context=context,kind='binned',transform=transform,population=population,policy=policy,control=control,oob=oob,inputs=encode(values),groups=as.list(as.character(groups)),levels=as.list(levels),seen=seen,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-facet-bin-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'faceted positional callback builds\n')
