# Development-only reference contract for figure tag grid placement.
library(ggplot2);library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2'))=='4.0.3',as.character(getRversion())=='4.6.1')
cases<-list()
for(location in c('plot','panel','margin'))for(position in c('topleft','top','topright','left','right','bottomleft','bottom','bottomright')){
 p<-ggplot(data.frame(x=1:3,y=1:3),aes(x,y))+geom_point()+labs(title='Title',caption='Caption',tag='A')+theme(plot.tag.location=location,plot.tag.position=position)
 result<-tryCatch({g<-ggplotGrob(p);list(tag=g$layout[g$layout$name=='tag',c('t','l','b','r','clip')],panel=g$layout[g$layout$name=='panel',c('t','l','b','r')],rows=length(g$heights),columns=length(g$widths))},error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-c(list(location=location,position=position),result)
}
invalid<-tryCatch({ggplotGrob(ggplot()+labs(tag='A')+theme(plot.tag.location='margin',plot.tag.position=c(.5,.5)));FALSE},error=function(e)TRUE)
writeLines(toJSON(list(reference=list(R=as.character(getRversion()),ggplot2=as.character(packageVersion('ggplot2'))),cases=cases,numeric_margin_rejected=invalid),auto_unbox=TRUE,pretty=TRUE,digits=17),'fixtures/parity/ggplot2/theme-furniture-controls.json')
cat('PASS',length(cases),'tag placement rows, numeric-margin rejection',invalid,'\n')
