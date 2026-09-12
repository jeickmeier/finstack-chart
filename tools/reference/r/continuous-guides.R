# FIX-GG04: continuous scale candidates retain outside values until labels are formatted.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
column<-function(x)lapply(seq_along(x),function(i){v<-x[[i]];if(is.na(v))return(NULL);if(is.numeric(v)&&!is.finite(v))return(if(v>0)'Infinity'else'-Infinity');unname(v)})
cases<-list()
for(trans in c('identity','sqrt','log10','reverse'))for(limits in list(c(1,10),c(.8,8.4),c(4,4)))for(bm in c('auto','explicit','empty'))for(lm in c('auto','hidden','explicit','short')) {
 breaks<-switch(bm,auto=waiver(),explicit=c(-1,0,.1,1,5,10,20,Inf,NA,NaN),empty=numeric())
 labels<-switch(lm,auto=waiver(),hidden=NULL,explicit=if(bm=='empty')character()else paste0('L',seq_len(if(bm=='explicit')10 else 5)),short=c('a','b'))
 result<-tryCatch({s<-scale_colour_gradient(limits=limits,transform=trans,breaks=breaks,labels=labels);b<-s$get_breaks();l<-s$get_labels(b);domain<-s$get_limits();list(domain=column(domain),breaks=column(b),labels=column(l),hidden=is.null(l),visible=as.list(is.finite(b)&b>=min(domain)&b<=max(domain)))},error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(transform=trans,limits=as.list(limits),break_mode=bm,label_mode=lm,result=result)
}
for(trans in c('identity','sqrt','log10','reverse'))for(n in c(3,7))for(limits in list(c(.8,8.4),c(1,1000))) {
 s<-scale_colour_gradient(limits=limits,transform=trans,n.breaks=n);b<-s$get_breaks();domain<-s$get_limits()
 cases[[length(cases)+1]]<-list(transform=trans,limits=as.list(limits),break_mode='auto',label_mode='auto',count=n,result=list(domain=column(domain),breaks=column(b),labels=column(s$get_labels(b)),hidden=FALSE,visible=as.list(is.finite(b)&b>=min(domain)&b<=max(domain))))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/continuous-guides.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'continuous guide records\n')
