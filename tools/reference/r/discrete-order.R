# FIX-GG04: character position training uses the pinned C collation; explicit limits retain order.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases<-list()
for(values in list(c('c','a','b','c'),c('b','NA','A','a','10','2'),c('z','b','a')))for(limits in list(NULL,c('z','c','b','a','NA','A','10','2'))){
 b<-ggplot_build(ggplot(data.frame(x=values,y=1),aes(x,y))+geom_point()+scale_x_discrete(limits=limits,minor_breaks=c(-2,.5,1,1.5,2,3,4)))
 x<-b$layout$panel_params[[1]]$x
 minor_position<-x$rescale(x$minor_breaks);minor_keep<-is.finite(minor_position)&is.finite(x$minor_breaks)
 cases[[length(cases)+1]]<-list(population='ordering',limits_name='default',inputs=as.list(values),levels=if(is.null(limits))NULL else as.list(limits),limits=NULL,expansion='default',result=list(labels=as.list(x$get_labels()),major_positions=as.list(unname(x$break_positions())),minor_values=as.list(unname(x$minor_breaks[minor_keep])),minor_positions=as.list(unname(minor_position[minor_keep])),point_positions=as.list(unname(x$rescale(b$data[[1]]$x)))))
}
jsonlite::write_json(list(cases=cases),'fixtures/parity/ggplot2/discrete-order.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete ordering records\n')
