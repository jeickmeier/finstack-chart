# Independent ggplot2 4.0.3 setup/draw controls for GG-07 row-grid surfaces.
library(ggplot2)
library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2'))=='4.0.3')
find_raster <- function(g) {
  if (!is.null(g$raster)) return(list(dim=dim(g$raster), pixels=as.vector(t(g$raster))))
  for (child in c(g$children,g$grobs)) {r<-find_raster(child);if(!is.null(r))return(r)}
  NULL
}
cases<-list()
for (hjust in c(0,0.5,1)) for(vjust in c(0,1)) {
 d<-data.frame(x=c(0,1,2.8),y=c(1,1,1),fill=c('red','green','blue'))
 p<-ggplot(d,aes(x,y,fill=fill))+geom_raster(hjust=hjust,vjust=vjust)+scale_fill_identity()
 warnings<-character()
 value<-withCallingHandlers({built<-ggplot_build(p);list(data=built$data[[1]],raster=find_raster(ggplotGrob(p)))},warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')})
 cases[[length(cases)+1]]<-list(hjust=hjust,vjust=vjust,input=d,data=value$data,raster=value$raster,warnings=unique(warnings))
}
writeLines(toJSON(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),auto_unbox=TRUE,pretty=TRUE,digits=17,na='null'),'fixtures/parity/ggplot2/surface-controls.json')
