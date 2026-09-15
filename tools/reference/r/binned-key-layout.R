pdf(file = tempfile(fileext = ".pdf"))
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases=list()
for(direction in c('vertical','horizontal'))for(reverse in c(FALSE,TRUE))for(show in c(FALSE,TRUE)) {
 p=ggplot(data.frame(x=1:4,v=c(1,4,9,16)),aes(x,1,size=v))+geom_point()+scale_size_binned(breaks=c(5,10),limits=c(0,20),guide=guide_bins(direction=direction,reverse=reverse,show.limits=show))
 b=ggplot_build(p);gt=ggplot_gtable(b);g=b$plot$guides$params[[1]]
 cases[[length(cases)+1]]=list(direction=direction,reverse=reverse,show_limits=show,labels=unname(g$key$.label),values=unname(g$key$.value),sizes=unname(g$key$size))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/binned-key-layout.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null')
