# Independent GG09 draw-component contract; numerical estimators are captured separately.
library(ggplot2)
library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2'))=='4.0.3')
d<-data.frame(g=factor(rep(c('A','B'),c(8,5))),y=c(0,0,1,2,3,3,4,20,1,2,3,4,5))
paint<-function(g){
 coords<-list();for(n in intersect(names(g),c('x','y','x0','y0','x1','y1','width','height','r','dotdia','stackposition','stackratio','stackdir','stackaxis'))){v<-g[[n]];coords[[n]]<-if(inherits(v,'unit'))as.numeric(v) else if(is.atomic(v))v else NULL}
 list(class=class(g)[1],gp=if(is.null(g$gp))NULL else unclass(g$gp)[intersect(names(g$gp),c('col','fill','lwd','lty','lineend','linejoin','fontsize'))],pch=g$pch,coords=coords,children=if(!is.null(g$children))unname(lapply(g$children,paint))else NULL)
}
cases<-list()
capture<-function(name,p,controls=list()){
 warnings<-character();v<-withCallingHandlers({b<-ggplot_build(p);g<-ggplotGrob(p);panel<-g$grobs[[which(g$layout$name=='panel')]];list(data=b$data[[1]],draw=unname(lapply(panel$children[grepl('geom_',names(panel$children))],paint)))},warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')})
 cases[[length(cases)+1]]<<-list(name=name,controls=controls,built=lapply(v$data,function(x)if(is.factor(x))as.character(x)else if(is.list(x))unname(x)else unclass(x)),draw=v$draw,warnings=unique(warnings))
}
b<-ggplot(d,aes(g,y))
capture('box-default',b+geom_boxplot())
capture('box-notched',b+geom_boxplot(notch=TRUE,notchwidth=.3,staplewidth=.7),list(notch=TRUE,notchwidth=.3,staplewidth=.7))
capture('box-varwidth',b+geom_boxplot(varwidth=TRUE),list(varwidth=TRUE))
capture('box-no-outliers',b+geom_boxplot(outliers=FALSE),list(outliers=FALSE))
capture('box-components',b+geom_boxplot(whisker.colour='red',whisker.linewidth=1,whisker.linetype=2,staplewidth=.5,staple.colour='blue',staple.linewidth=.25,median.colour='green',median.linewidth=2,box.colour='purple',box.linewidth=.75,fill='gold',alpha=.2,outlier.colour='orange',outlier.fill='blue',outlier.shape=21,outlier.size=3,outlier.stroke=1,outlier.alpha=.6))
capture('box-horizontal',ggplot(d,aes(y,g))+geom_boxplot(orientation='y',notch=TRUE,staplewidth=.5),list(orientation='y'))
for(norm in c('area','count','width'))capture(paste0('violin-',norm),b+geom_violin(scale=norm),list(scale=norm))
capture('violin-quantiles',b+geom_violin(quantiles=c(.25,.5,.75),quantile.linetype=2,quantile.colour='red',quantile.linewidth=1,fill='gold',alpha=.2),list(quantiles=c(.25,.5,.75)))
capture('violin-untrimmed',b+geom_violin(trim=FALSE),list(trim=FALSE))
capture('violin-horizontal',ggplot(d,aes(y,g))+geom_violin(orientation='y'),list(orientation='y'))
dd<-data.frame(x=c(0,0,.2,1,1.2,2,2,2),g=factor(c('A','B','A','A','B','A','B','B')))
for(axis in c('x','y'))for(direction in c('up','down','center','centerwhole')){
 p<-if(axis=='x')ggplot(dd,aes(x,fill=g)) else ggplot(dd,aes(g,x,fill=g))
 capture(paste('dot',axis,direction,sep='-'),p+geom_dotplot(binwidth=.5,binaxis=axis,stackdir=direction,stackratio=.8,dotsize=.7),list(binaxis=axis,stackdir=direction,stackratio=.8,dotsize=.7))
}
capture('dot-stackgroups',ggplot(dd,aes(x,fill=g))+geom_dotplot(binwidth=.5,stackgroups=TRUE,binpositions='all'),list(stackgroups=TRUE,binpositions='all'))
capture('density-default',ggplot(dd,aes(x))+geom_density())
for(outline in c('upper','lower','both','full')) capture(paste0('density-',outline),ggplot(dd,aes(x))+geom_density(outline.type=outline,fill='gold',alpha=.2),list(outline=outline))
writeLines(toJSON(list(reference='ggplot2 4.0.3 / R 4.6.1',input=d,dot_input=dd,cases=cases),auto_unbox=TRUE,pretty=FALSE,digits=17,na='null'),'fixtures/parity/ggplot2/distribution-geometry-controls.json')
