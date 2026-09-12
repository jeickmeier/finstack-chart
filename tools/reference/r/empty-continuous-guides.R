# FIX-GG04: zero-row population is distinct from a fallback mapping domain.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases<-list()
for(family in c('continuous','identity'))for(lm in c('none','partial','full'))for(bm in c('auto','explicit','empty')) {
 limits<-switch(lm,none=NULL,partial=c(NA,10),full=c(1,10))
 breaks<-switch(bm,auto=waiver(),explicit=c(1,2),empty=numeric())
 s<-if(family=='identity')scale_size_identity(limits=limits,breaks=breaks,guide='legend')else scale_colour_gradient(limits=limits,breaks=breaks)
 s$train(numeric())
 b<-s$get_breaks()
 cases[[length(cases)+1]]<-list(family=family,limits=lm,breaks=bm,values=as.list(unname(b)),labels=as.list(unname(s$get_labels(b))))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/empty-continuous-guides.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'empty continuous guide records\n')
