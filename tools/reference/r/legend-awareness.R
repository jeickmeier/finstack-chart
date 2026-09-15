pdf(file = tempfile(fileext = ".pdf"))
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases=list()
for(force in list(NA,TRUE,FALSE,c(shape=FALSE),c(colour=TRUE),c(colour=FALSE))) {
 d=data.frame(x=c(1,2),g=c('A','B'))
 p=ggplot(d,aes(x,1,colour=g))+geom_point(data=d[1,,drop=FALSE],show.legend=force)+geom_point(data=d[2,,drop=FALSE],shape=17)+scale_colour_discrete(labels=c('Alpha','Beta'))
 b=ggplot_build(p);gt=ggplot_gtable(b)
 g=b$plot$guides$params[[1]]
 cases[[length(cases)+1]]=list(force=if(is.null(names(force)))force else as.list(force),labels=unname(g$key$.label),draw=lapply(g$decor,function(d)unname(d$data$.draw)))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/legend-awareness.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null')
