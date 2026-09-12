# FIX-GG04: explicit date_labels on actual hms scale panels.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
Sys.setenv(TZ='UTC'); Sys.setlocale('LC_TIME','C'); options(digits.secs=0)
library(ggplot2)
patterns<-c('%H:%M:%S','%F %T %OS6','%OS0|%OS3|%OS6','%OS1','%OS3','%OS6','%c|%x|%X','%C|%Y|%G|%j|%V','%z|%Z','%n%t%H|%q|%%OS3')
domains<-list(c(-90061.123456,-0.1),c(-0.1,.1),c(5.123456,86400.123456),c(1e12,1e12+1))
cases<-list()
for(domain in domains)for(pattern in patterns){
 x<-hms::as_hms(domain)
 p<-ggplot(data.frame(x=x,y=c(0,1)),aes(x,y))+geom_point()+scale_x_time(limits=x,breaks=x,date_labels=pattern,expand=expansion(0))
 axis<-ggplot_build(p)$layout$panel_params[[1]]$x
 keep<-!is.na(axis$breaks)
 cases[[length(cases)+1]]<-list(seconds=as.list(sprintf('%.17g',domain)),pattern=pattern,labels=as.list(unname(axis$get_labels()[keep])))
}
jsonlite::write_json(list(reference='R 4.6.1 / ggplot2 4.0.3; LC_TIME=C; digits.secs=0; TZ=UTC',cases=cases),'fixtures/parity/ggplot2/duration-formats.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'duration format panels\n')
