# FIX-GG04: binned positional functions drive both classification and panel guides.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(x)lapply(unname(x),function(v)if(is.na(v))NULL else if(is.numeric(v)&&!is.finite(v))if(v>0)'Infinity'else'-Infinity'else v)
cases<-list()
for(tr in c('identity','sqrt','log10','reverse')) for(pop in c('ordinary','constant','missing','all_missing','empty'))
 for(limits in c('none','full')) for(show_limits in c(FALSE,TRUE))
  for(signature in c('limits','n','n.breaks')) for(count in c('default','three','zero'))
   for(mode in c('domain','mixed','empty','null')) for(label_mode in c('indexed','automatic')) {
    values<-switch(pop,ordinary=c(1,4,10),constant=c(4,4),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric());calls<-list();label_calls<-list()
    evaluate<-function(x,provided=NULL,effective=NULL) {
     calls[[length(calls)+1L]]<<-list(limits=encode(x),names=as.list(names(x)),count=provided,effective=effective)
     switch(mode,domain=x,mixed=setNames(c(x[2],mean(x),x[1],x[1],NA,Inf,-Inf),c('last','middle','first','again','missing','positive','negative')),empty=numeric(),null=NULL)
    }
    breaks<-switch(signature,limits=function(x)evaluate(x),n=function(x,n=7)evaluate(x,if(missing(n))NULL else n,n),n.breaks=function(x,n.breaks=9)evaluate(x,if(missing(n.breaks))NULL else n.breaks,n.breaks))
    labels<-function(x) {label_calls[[length(label_calls)+1L]]<<-list(values=encode(x),names=as.list(names(x)));paste0(seq_along(x),'/',length(x))}
    result<-tryCatch(suppressWarnings({
     s<-scale_x_binned(transform=tr,limits=if(limits=='full')c(1,10)else NULL,show.limits=show_limits,breaks=breaks,n.breaks=switch(count,default=10,three=3,zero=0),labels=if(label_mode=='indexed')labels else waiver())
     b<-ggplot_build(ggplot(data.frame(v=values,y=rep(1,length(values))),aes(v,y))+geom_point()+s)
     panel<-b$layout$panel_params[[1]]$x
     list(values=encode(panel$get_breaks()),labels=encode(panel$get_labels()),range=encode(panel$continuous_range),mapped=encode(b$data[[1]]$x))
    }),error=function(e)list(error=conditionMessage(e)))
    cases[[length(cases)+1L]]<-list(transform=tr,population=pop,limits=limits,show_limits=show_limits,signature=signature,count=count,mode=mode,label_mode=label_mode,inputs=encode(values),calls=calls,label_calls=label_calls,result=result)
   }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-binned-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'binned positional break function builds\n')
