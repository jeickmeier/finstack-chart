library(ggplot2);library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2'))=='4.0.3')
cases<-list()
for(r in list(c(0,8),c(-1,1),c(.001,.0019),c(2,2),c(0,0),c(100,100.2),c(.12,.99))){
 for(n in c(NA,1,3,7)){result<-tryCatch(list(breaks=ggplot2:::contour_breaks(r,bins=if(is.na(n))NULL else n)),error=function(e)list(error=conditionMessage(e)));cases[[length(cases)+1]]<-c(list(range=r,bins=n),result)}
}
labels <- lapply(list(c(.5,2,4),c(.00012345,.00023456,.00034567),c(1,1.00001,1.00002)),function(b)list(breaks=b,labels=ggplot2:::pretty_isoband_levels(paste(head(b,-1),tail(b,-1),sep=':'))))
writeLines(toJSON(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases,labels=labels),auto_unbox=TRUE,digits=17,na='null'),'fixtures/parity/ggplot2/spatial-contour-levels.json')
