# FIX-GG04: scale binned candidates/labels, before GuideColoursteps composition.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases<-list()
for(tr in c('identity','sqrt','log10','reverse'))for(domain in list(c(1,10),c(.8,8.4),c(4,4)))for(lm in c('none','full'))for(bm in c('nice','equal','explicit','empty'))for(label_mode in c('auto','hidden','explicit')) {
 breaks<-switch(bm,nice=waiver(),equal=waiver(),explicit=c(0,1,1,3,20),empty=numeric())
 labels<-switch(label_mode,auto=waiver(),hidden=NULL,explicit=if(bm=='empty')character()else paste0('L',1:5))
 result<-tryCatch({
  s<-scale_colour_steps(transform=tr,limits=if(lm=='full')domain else NULL,breaks=breaks,labels=labels,n.breaks=5,nice.breaks=bm!='equal')
  s$train(s$transform(domain)); b<-s$get_breaks(); labels<-s$get_labels(b)
  list(breaks=as.list(unname(b)),labels=as.list(unname(labels)),hidden=is.null(labels),limits=as.list(unname(s$get_limits())),visible=as.list(is.finite(scales::oob_censor_any(b,s$get_limits()))))
 },error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(transform=tr,domain=as.list(domain),limits=lm,breaks=bm,labels=label_mode,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/binned-guides.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null',na='string')
cat('PASS',length(cases),'binned candidate/label records\n')
