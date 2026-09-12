# FIX-GG04: scale break intersection, original label positions and named replacement.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
column<-function(x)lapply(seq_along(x),function(i)if(is.na(x[[i]]))NULL else unname(x[[i]]))
break_sets<-list(auto=waiver(),selected=c('c','z','a','c',NA),empty=character(),named=c(Third='c',Outside='z',First='a',Again='c',Missing=NA))
label_sets<-list(auto=waiver(),hidden=NULL,explicit=c('C','Z','A','C2','Missing'),named=c(c='cee',z='zed',a='aye',c='last',unused='unused'),short=c('first','second'))
cases<-list()
for(kind in c('hue','manual','identity'))for(b in names(break_sets))for(l in names(label_sets)){
 breaks<-break_sets[[b]];labels<-label_sets[[l]]
 result<-tryCatch({
  args<-list(limits=c('a','b','c',NA),breaks=breaks,labels=labels)
  if(kind=='manual')args$values<-c('#FF0000','#00FF00','#0000FF')
  if(kind=='identity')args$guide<-'legend'
  s<-do.call(get(paste0('scale_colour_',kind),asNamespace('ggplot2')),args)
  s$train(c('a','b','c',NA));br<-s$get_breaks();la<-s$get_labels(br)
  list(breaks=column(br),labels=column(la),hidden=is.null(la))
 },error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,break_mode=b,label_mode=l,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/discrete-guides.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete guide records\n')
