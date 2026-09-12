# FIX-GG04: authored continuous limits change category-index expansion, not the trained labels.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.nan(x))'NaN'else if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else unname(x))
populations<-list(empty=list(values=character()),single=list(values='b'),three=list(values=c('a','b','c')),unused=list(values=c('a','c'),levels=letters[1:4]),reordered=list(values=c('a','c'),levels=c('c','b','a','d')),missing=list(values=c('a','z'),levels=letters[1:3]),all_outside=list(values='z',levels=letters[1:3]),empty_levels=list(values=character(),levels=letters[1:3]),no_levels=list(values=c('a','c'),levels=character()))
limits_list<-list(default=NULL,wide=c(0,5),narrow=c(1.5,2.5),reverse=c(5,0),constant=c(2,2),missing_left=c(NA,5),missing_right=c(0,NA),missing_both=c(NA_real_,NA_real_),unbounded=c(-Inf,Inf),left_inf=c(-Inf,4),right_inf=c(0,Inf),same_inf=c(Inf,Inf),same_neg_inf=c(-Inf,-Inf),far=c(10,12),singleton=2,triple=c(-2,1,4),empty=numeric(),missing_short=NA_real_,missing_long=c(NA_real_,2,5),logical=c(TRUE,FALSE),invalid_type=c("a","b"))
cases<-list()
for(population_name in names(populations))for(limit_name in names(limits_list))for(expand_name in c('default','none','asymmetric','contract')){
 population<-populations[[population_name]];limits<-limits_list[[limit_name]]
 expansion_value<-switch(expand_name,default=waiver(),none=expansion(0),asymmetric=expansion(mult=c(.1,.2),add=c(.3,.7)),contract=expansion(mult=c(-.5,-.25),add=c(-.2,-.1)))
 result<-tryCatch(suppressWarnings({
  p<-ggplot(data.frame(x=population$values,y=seq_along(population$values)),aes(x,y))+geom_point()+scale_x_discrete(limits=population$levels,continuous.limits=limits,expand=expansion_value,minor_breaks=c(-2,.5,1,1.5,2,3,4))
  b<-ggplot_build(p);view<-b$layout$panel_params[[1]];x<-view$x
  major_position<-x$break_positions();major_keep<-is.finite(major_position)
  minor_position<-x$rescale(x$minor_breaks);minor_keep<-is.finite(minor_position)&is.finite(x$minor_breaks)
  list(range=encode(x$continuous_range),mapped=encode(b$data[[1]]$x),point_positions=encode(b$layout$coord$transform(b$data[[1]],view)$x),labels=as.list(unname(x$get_labels()[major_keep])),major_positions=encode(major_position[major_keep]),minor_values=encode(x$minor_breaks[minor_keep]),minor_positions=encode(minor_position[minor_keep]))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(population=population_name,inputs=as.list(population$values),levels=if(is.null(population$levels))NULL else as.list(population$levels),limits_name=limit_name,limits=if(is.null(limits))NULL else encode(limits),expansion=expand_name,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/discrete-continuous-limits.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete continuous-limit panels\n')
