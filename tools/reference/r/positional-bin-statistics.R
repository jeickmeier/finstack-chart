# FIX-GG04: positional bins feed indices into grouped statistics before interval mapping.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(v,function(x) if(is.nan(x)) 'NaN' else if(is.na(x)) NULL else if(is.infinite(x)) if(x>0) 'Infinity' else '-Infinity' else unname(x))
cases <- list()
for(trans in c('identity','sqrt','log10','reverse')) for(mode in c('nice','equal','explicit')) for(right in c(TRUE,FALSE)) for(grouped in c(FALSE,TRUE)) for(filtered in c(FALSE,TRUE)) {
 d<-data.frame(x=1,y=c(1,2,4,5,8,10),g=rep(c('A','B'),3))
 if(filtered) d<-d[d$y>2,]
 p<-ggplot(d,aes(x,y))+stat_summary(fun=mean,geom='point',mapping=if(grouped)aes(group=g)else NULL)+scale_y_binned(transform=trans,n.breaks=3,nice.breaks=mode!='equal',breaks=if(mode=='explicit')c(1,2,4,10)else waiver(),right=right)
 result<-tryCatch(suppressWarnings({b<-ggplot_build(p);list(x=encode(b$data[[1]]$x),y=encode(b$data[[1]]$y))}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(transform=trans,mode=mode,right=right,grouped=grouped,filtered=filtered,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-bin-statistics.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'positional bin statistic cases\n')
