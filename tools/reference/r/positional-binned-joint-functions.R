# FIX-GG04: positional limit selectors before and after registered bin cuts.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
dimensions <- "--dimensions" %in% commandArgs(trailingOnly=TRUE)
pdf(file=tempfile(fileext=".pdf"))
encode<-function(x)lapply(unname(x),function(v)if(is.na(v))NULL else if(is.numeric(v)&&!is.finite(v))if(v>0)'Infinity'else'-Infinity'else v)
cases<-list()
for(tr in c('identity','sqrt','log10','reverse')) for(pop in c('ordinary','constant','missing','all_missing','empty'))
 for(show_limits in c(FALSE,TRUE)) for(control in c('identity','reverse','fixed','lower_zero','missing_lower','empty','single'))
  for(mode in c('domain','mixed')) for(label_mode in c('indexed','automatic')) {
   values<-switch(pop,ordinary=c(1,4,10),constant=c(4,4),missing=c(NA,1,10),all_missing=c(NA_real_,NA_real_),empty=numeric())
   limit_calls<-list();calls<-list();label_calls<-list()
   limits<-function(x) {
    limit_calls[[length(limit_calls)+1L]]<<-encode(x)
    switch(control,identity=x,reverse=rev(x),fixed=c(1,10),lower_zero=c(0,x[2]),missing_lower=c(NA_real_,x[2]),empty=numeric(),single=5)
   }
   breaks<-function(x,n=7) {
    calls[[length(calls)+1L]]<<-list(limits=encode(x),names=as.list(names(x)),count=if(missing(n))NULL else n,effective=n)
    if(mode=='domain')x else setNames(c(x[2],mean(x),x[1],x[1],NA,Inf,-Inf),c('last','middle','first','again','missing','positive','negative'))
   }
   labels<-function(x) {label_calls[[length(label_calls)+1L]]<<-list(values=encode(x),names=as.list(names(x)));paste0(seq_along(x),'/',length(x))}
   result<-tryCatch(suppressWarnings({
    s<-scale_x_binned(transform=tr,limits=limits,show.limits=show_limits,breaks=breaks,n.breaks=3,labels=if(label_mode=='indexed')labels else waiver())
    b<-ggplot_build(ggplot(data.frame(v=values,y=rep(1,length(values))),aes(v,y))+geom_point()+s)
    panel<-b$layout$panel_params[[1]]$x
    result <- list(values=encode(panel$get_breaks()),labels=encode(panel$get_labels()),range=encode(panel$continuous_range),mapped=encode(b$data[[1]]$x))
    if(dimensions) result$positions <- tryCatch(encode(panel$break_positions()),error=function(e)list(error=conditionMessage(e)))
    result
   }),error=function(e)list(error=conditionMessage(e)))
   cases[[length(cases)+1L]]<-list(transform=tr,population=pop,control=control,show_limits=show_limits,mode=mode,label_mode=label_mode,signature='n',count='three',inputs=encode(values),limit_calls=limit_calls,calls=calls,label_calls=label_calls,result=result)
  }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),if(dimensions)'fixtures/parity/ggplot2/positional-binned-joint-dimensions.json'else'fixtures/parity/ggplot2/positional-binned-joint-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'joint positional binned function reference builds\n')

dev.off()
