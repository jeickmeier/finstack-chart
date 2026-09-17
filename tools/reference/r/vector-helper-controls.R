# Development-only independent oracle; no generated fixture is needed at runtime.
library(ggplot2)
library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2'))=='4.0.3',as.character(getRversion())=='4.6.1')
cases<-list()
add<-function(.case,.operation,.x,...){args<-list(...);result<-tryCatch({v<-do.call(get(.operation,asNamespace('ggplot2')),c(list(x=.x),args));if(is.factor(v))list(codes=I(unname(as.integer(v))-1L),levels=I(levels(v)),ordered=is.ordered(v))else list(value=v)},error=function(e)list(error=conditionMessage(e)));cases[[length(cases)+1]]<<-list(name=.case,operation=.operation,x=.x,integer=is.integer(.x),options=args,result=result)}
for(right in c(TRUE,FALSE)){
 add(paste0('interval-',right),'cut_interval',c(-3,-2,-1,0,1,2,3,NA),n=3,right=right)
 add(paste0('number-',right),'cut_number',c(0,1,2,4,8,16,NA),n=3,right=right)
 add(paste0('width-',right),'cut_width',c(-2,-1,-.5,0,.5,1,2,NA),width=1,closed=if(right)'right'else'left')
}
add('interval-length','cut_interval',c(-1.2,0,.3,2.6),length=.5)
add('interval-custom-labels','cut_interval',c(0,1,2,3),n=3,labels=c('a','b','c'),ordered_result=TRUE)
add('width-center','cut_width',c(-3,-2,-1,0,1,2,3),width=2,center=0)
add('width-boundary','cut_width',c(-3,-2,-1,0,1,2,3),width=2,boundary=0)
add('interval-single','cut_interval',c(0,1,2),n=1)
add('interval-duplicate-labels','cut_interval',c(0,1,2,3),n=3,labels=c('a','a','b'))
add('interval-close-labels','cut_interval',c(1,1.00001,1.00002),n=2)
add('interval-scientific','cut_interval',c(1e-6,2e-6,3e-6),n=2)
add('number-ties','cut_number',c(1,1,1,2),n=3)
add('interval-constant','cut_interval',c(1,1,1),n=2)
add('empty','cut_interval',numeric(),n=2)
add('width-invalid','cut_width',1:3,width=0)
add('width-both','cut_width',1:3,width=1,center=0,boundary=0)
for(z in c(TRUE,FALSE))for(i in seq_along(list(c(1.1,2.3,3.5),c(4,4),c(NA,1,3),c(1e-12,2e-12,3e-12),c(1000000,1000000+1e-9),numeric(),1:4,c(1,1+1e-9,3)))){x<-list(c(1.1,2.3,3.5),c(4,4),c(NA,1,3),c(1e-12,2e-12,3e-12),c(1000000,1000000+1e-9),numeric(),1:4,c(1,1+1e-9,3))[[i]];add(paste('resolution',z,i),'resolution',x,zero=z)}
writeLines(toJSON(list(reference=list(R=as.character(getRversion()),ggplot2=as.character(packageVersion('ggplot2'))),cases=cases),auto_unbox=TRUE,pretty=TRUE,digits=17,na='null'),'fixtures/parity/ggplot2/vector-helper-controls.json')
cat('PASS',length(cases),'vector helper cases\n')
