# Independent GG07 interval component styling from pinned ggplot2 build/draw.
library(ggplot2)
library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2'))=='4.0.3')
d<-data.frame(x=1,y=2,lo=1,hi=3)
paint<-function(g){
 gp<-if(is.null(g$gp))NULL else unclass(g$gp)[intersect(names(g$gp),c('col','fill','lwd','lty','lineend','linejoin','fontsize'))]
 list(class=class(g)[1],gp=gp,pch=g$pch,children=if(!is.null(g$children))lapply(g$children,paint)else NULL)
}
cases<-list()
capture<-function(name,geom,controls=list()){
 warnings<-character()
 v<-withCallingHandlers({p<-ggplot(d,aes(x,y,ymin=lo,ymax=hi))+geom;b<-ggplot_build(p);g<-ggplotGrob(p);panel<-g$grobs[[which(g$layout$name=='panel')]];list(data=b$data[[1]],draw=lapply(panel$children[grepl('geom_',names(panel$children))],paint))},warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')})
 cases[[length(cases)+1]]<<-list(name=name,controls=controls,data=v$data,draw=v$draw,warnings=unique(warnings))
}
capture('crossbar-default',geom_crossbar())
capture('crossbar-alpha',geom_crossbar(alpha=0.2),list(alpha=0.2))
capture('crossbar-components',geom_crossbar(middle.colour='red',middle.linetype=2,middle.linewidth=2,box.colour='blue',box.linetype=3,box.linewidth=0.25,fill='gold',alpha=0.2),list(middle_colour='red',middle_linetype=2,middle_linewidth=2,box_colour='blue',box_linetype=3,box_linewidth=0.25,fill='gold',alpha=0.2))
capture('crossbar-embedded-alpha',geom_crossbar(colour='#FF000080',alpha=0.2,fill='#FFD70080'),list(colour='#FF000080',alpha=0.2,fill='#FFD70080'))
capture('crossbar-fatten',geom_crossbar(fatten=1),list(fatten=1))
capture('pointrange-default',geom_pointrange())
capture('pointrange-point-controls',geom_pointrange(size=1,stroke=2,shape=21,fill='gold',linewidth=0.25),list(size=1,stroke=2,shape=21,fill='gold',linewidth=0.25))
capture('pointrange-fatten',geom_pointrange(fatten=2),list(fatten=2))
for(cap in c('round','square'))capture(paste0('pointrange-',cap),geom_pointrange(lineend=cap),list(lineend=cap))
writeLines(toJSON(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),auto_unbox=TRUE,pretty=TRUE,digits=17,na='null'),'fixtures/parity/ggplot2/interval-paint-controls.json')
