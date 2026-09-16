# GG09 source anchors. Runtime code never loads R or executes host callbacks.
library(ggplot2)
library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
add <- function(name, family, input, params, plot) {
  warnings <- character()
  value <- withCallingHandlers(tryCatch(ggplot_build(plot)$data[[1]], error=function(e) list(error=conditionMessage(e))),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')})
  cases[[length(cases)+1]] <<- list(name=name,family=family,input=input,params=params,rows=value,warnings=warnings)
}
for (weighted in c(FALSE,TRUE)) for (pad in c(FALSE,TRUE)) for (grid in c(FALSE,TRUE)) {
 d <- data.frame(x=c(3,1,2,2),w=c(2,1,0,3))
 a <- if(weighted) aes(x=x,weight=w) else aes(x=x)
 add(paste('ecdf',weighted,pad,grid,sep='-'),'ecdf',d,list(weighted=weighted,pad=pad,n=if(grid)5 else NULL),ggplot(d,a)+stat_ecdf(pad=pad,n=if(grid)5 else NULL))
}
for (sample in list(c(1),c(1,2,3,4),c(-2,0,0,1,3,5,9),c(2,2,2))) {
 d <- data.frame(sample=sample)
 for (distribution in c('normal','uniform','logistic')) {
  fun <- switch(distribution,normal=qnorm,uniform=qunif,logistic=qlogis)
  add(paste('qq',distribution,length(sample),sep='-'),'qq',d,list(distribution=distribution),ggplot(d,aes(sample=sample))+stat_qq(distribution=fun))
  add(paste('qq-line',distribution,length(sample),sep='-'),'qq_line',d,list(distribution=distribution),ggplot(d,aes(sample=sample))+stat_qq_line(distribution=fun))
 }
}
for (log_x in c(FALSE,TRUE)) {
 d <- data.frame(x=c(1,100))
 p <- ggplot(d,aes(x=x))+stat_function(fun=function(x)x*x,n=5)
 if(log_x)p<-p+scale_x_log10()
 add(paste('function-square',log_x,sep='-'),'function',d,list(n=5,expression='x*x',log_x=log_x),p)
}
d<-data.frame(x=c(1,1,1,2),y=c(2,2,3,4),colour=c('A','A','A','B'))
add('unique-aesthetics','unique',d,list(),ggplot(d,aes(x,y,colour=colour))+stat_unique())
for (duplicate in c(FALSE,TRUE)) {
 d<-if(duplicate)data.frame(x=c(0,1,1,3,0,2,3),y=c(0,2,-1,1,1,-2,2),g=c('A','A','A','A','B','B','B')) else data.frame(x=c(0,1,3,0,2,3),y=c(0,2,-1,1,-2,2),g=c('A','A','A','B','B','B'))
 add(paste('align',duplicate,sep='-'),'align',d,list(),ggplot(d,aes(x,y,group=g))+stat_align(position='identity'))
}
for (weights in list(c(1,-2,3),c(0,0,0),c(1,NA,Inf),c(1,-1,1e-15))) {
 d<-data.frame(x=c(1,2,3),w=weights)
 add(paste('ecdf-weight-boundary',length(cases),sep='-'),'ecdf',d,list(weighted=TRUE,pad=FALSE),ggplot(d,aes(x=x,weight=w))+stat_ecdf(pad=FALSE))
}
for (connection in list('hv','vh','mid',matrix(c(0,0,0.25,0.75,1,1),ncol=2,byrow=TRUE))) {
 d<-data.frame(x=c(3,1,2),y=c(2,-1,4))
 add(paste('connect',length(cases),sep='-'),'connect',d,list(connection=connection),ggplot(d,aes(x,y))+stat_connect(connection=connection))
}
write_json(list(reference=list(ggplot2=as.character(packageVersion('ggplot2')),R=as.character(getRversion())),cases=cases),'fixtures/parity/ggplot2/univariate-controls.json',pretty=TRUE,auto_unbox=TRUE,digits=17,na='string',null='null')
cat('PASS',length(cases),'univariate source cases\n')
