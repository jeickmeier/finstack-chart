# Independent GG07 lineend/linejoin and device-unit controls from pinned ggplot2.
library(ggplot2)
library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2'))=='4.0.3')
d<-data.frame(x=c(1,2,2.1,3),y=c(1,3,1,2))
paint<-function(g)list(class=class(g)[1],gp=if(is.null(g$gp))NULL else unclass(g$gp)[intersect(names(g$gp),c('col','fill','lwd','lty','lineend','linejoin','linemitre'))],children=if(!is.null(g$children))lapply(g$children,paint)else NULL)
cases<-list()
for(end in c('butt','round','square'))for(join in c('mitre','round','bevel')){
 p<-ggplot(d,aes(x,y))+geom_path(linewidth=2,alpha=0.5,linetype=2,lineend=end,linejoin=join)
 g<-ggplotGrob(p);panel<-g$grobs[[which(g$layout$name=='panel')]]
 cases[[length(cases)+1]]<-list(lineend=end,linejoin=join,draw=lapply(panel$children[grepl('GRID.polyline',names(panel$children))],paint))
}
writeLines(toJSON(list(reference='ggplot2 4.0.3 / R 4.6.1',grid_defaults=unclass(grid::get.gpar(c('lineend','linejoin','linemitre'))),cases=cases),auto_unbox=TRUE,pretty=TRUE,digits=17),'fixtures/parity/ggplot2/stroke-controls.json')
