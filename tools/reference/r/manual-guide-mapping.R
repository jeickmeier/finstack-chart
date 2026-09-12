# FIX-GG04: authored manual breaks name an otherwise unnamed palette.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
column<-function(x)lapply(seq_along(x),function(i)if(is.na(x[[i]]))NULL else unname(x[[i]]))
cases<-list()
for(named in c(FALSE,TRUE))for(br in list(waiver(),c('c','a'),c('c','z','a','c',NA),character(),c('c','c','a')))for(lim in list(NULL,c('a','b','c'),c('a','b','c','d'))) {
 values<-c('red','green','blue');if(named)names(values)<-c('c','c','a')
 result<-tryCatch({s<-scale_colour_manual(values=values,breaks=br,limits=lim);s$train(c('a','b','c',NA));list(domain=column(s$get_limits()),mapped=column(s$map(c('a','b','c','d',NA))))},error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(named=named,breaks=if(is_waiver(br))NULL else column(br),automatic=is_waiver(br),limits=if(is.null(lim))NULL else column(lim),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/manual-guide-mapping.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'manual break mapping records\n')
