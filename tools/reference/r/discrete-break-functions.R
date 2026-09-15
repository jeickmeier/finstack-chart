# FIX-GG04: discrete break functions, coercion, names and label selection.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode <- function(x) lapply(unname(x),function(v)if(is.na(v))NULL else v)
cases <- list()
for(channel in c('colour','shape','size','alpha','linewidth','linetype'))
 for(population in c('ordinary','nullable','all_missing','empty','numeric_text'))
  for(limits in c('trained','explicit'))
   for(mode in c('domain','mixed','empty','null','numeric'))
    for(label_mode in c('indexed','default')) {
    values <- switch(population,ordinary=c('a','b','c'),nullable=c('a','b',NA),all_missing=c(NA_character_,NA_character_),empty=character(),numeric_text=c('1','2',NA))
    calls <- list(); label_calls <- list()
    breaks <- function(x) {
     calls[[length(calls)+1L]] <<- list(values=encode(x),names=as.list(names(x)))
     switch(mode,domain=x,mixed=c(Z='z',B='b',B2='b',M=NA,A='a'),empty=character(),null=NULL,numeric=c(one=1,missing=NA,again=1))
    }
    labels <- function(x) {
     label_calls[[length(label_calls)+1L]] <<- list(values=encode(x),names=as.list(names(x)))
     paste0(seq_along(x),'/',length(x))
    }
    result <- tryCatch(suppressWarnings({
     constructor <- get(paste0('scale_',channel,'_discrete'))
     scale <- constructor(limits=if(limits=='explicit')c('c','b','a',NA)else NULL,breaks=breaks,labels=if(label_mode=='default')waiver()else labels)
     mapping <- aes(x,y,group=1);mapping[[channel]] <- quote(v)
     geometry <- if(channel%in%c('linewidth','linetype'))geom_line()else geom_point()
     built <- ggplot_build(ggplot(data.frame(x=seq_along(values),y=seq_along(values),v=values),mapping)+geometry+scale)
     list(keys=unname(lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label)))))
    }),error=function(e)list(error=conditionMessage(e)))
    cases[[length(cases)+1L]] <- list(channel=channel,population=population,limits=limits,mode=mode,label_mode=label_mode,inputs=encode(values),calls=calls,label_calls=label_calls,result=result)
   }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/discrete-break-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete break function reference builds\n')
