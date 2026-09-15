# FIX-GG04: date_breaks/date_labels override registered functions independently.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
default_names <- "--default-names" %in% commandArgs(trailingOnly=TRUE)
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.numeric(v)&&!is.finite(v)) if(v>0)'Infinity' else '-Infinity' else v)
metadata <- function(x) list(values=encode(as.numeric(x)),class=as.list(class(x)),zone=as.list(attr(x,'tzone')),names=as.list(names(x)))
cases <- list()
for(channel in c('colour','size')) for(kind in c('date','datetime'))
 for(zone in if(kind=='date')'UTC' else c('UTC','America/New_York'))
  for(epoch in if(kind=='date')1704067200 else c(1710046800,1730606400))
   for(control in c('width','format','both')) {
    typed <- function(v) if(kind=='date')as.Date(v,origin='1970-01-01')else as.POSIXct(v,origin='1970-01-01',tz=zone)
    values<-typed(if(kind=='date')epoch/86400+c(0,1,2,3)else epoch+c(0,3600,7200,10800))
    calls<-list();label_calls<-list()
    breaks<-function(x,n=7) {
     calls[[length(calls)+1L]]<<-c(metadata(x),list(count=if(missing(n))NULL else n,effective=n))
     v<-as.numeric(x)
     setNames(typed(c(v[2],mean(v),v[1],v[1],NA,Inf,-Inf)),c('last','middle','first','again','missing','positive','negative'))
    }
    labels<-function(x) {label_calls[[length(label_calls)+1L]]<<-metadata(x);paste0(seq_along(x),'/',length(x))}
    result<-tryCatch(suppressWarnings({
     args<-list(breaks=breaks,labels=if(default_names)waiver()else labels,n.breaks=3)
     if(control!='format')args$date_breaks<-if(kind=='date')'1 day'else'1 hour'
     if(control!='width')args$date_labels<-if(kind=='date')'%Y-%m-%d'else'%H:%M'
     if(kind=='datetime')args$timezone<-zone
     if(channel=='size')args$range<-c(1,6)
     scale<-do.call(get(paste0('scale_',channel,'_',kind)),args)
     mapping<-aes(x,1);mapping[[channel]]<-quote(v)
     built<-ggplot_build(ggplot(data.frame(x=seq_along(values),v=values),mapping)+geom_point()+scale)
     list(keys=unname(lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label)))),mapped=encode(built$data[[1]][[channel]]))
    }),error=function(e)list(error=conditionMessage(e)))
    cases[[length(cases)+1L]]<-list(channel=channel,kind=kind,zone=zone,epoch=epoch,control=control,population='spaced',limits='none',signature='n',count='three',mode='mixed',inputs=encode(as.numeric(values)),calls=calls,label_calls=label_calls,result=result)
   }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),if(default_names)'fixtures/parity/ggplot2/temporal-break-default-overrides.json'else'fixtures/parity/ggplot2/temporal-break-overrides.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'temporal break override reference builds\n')
