# FIX-GG04: primary plot outcomes are separate from standalone scale mapping.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
base<-jsonlite::read_json('fixtures/parity/ggplot2/limit-functions.json')$cases
cases<-list()
for(c in base) {
 if(is.null(c$transform))next
 inputs<-vapply(c$inputs,function(v)if(is.null(v))NA_real_ else if(is.character(v))as.numeric(v) else v,numeric(1))
 control<-c$control;seen<-list()
 fun<-function(x){seen[[length(seen)+1L]]<<-encode(x);switch(control,identity=x,reverse=rev(x),fixed=c(0,10),lower_zero=c(0,x[2]),missing_lower=c(NA_real_,x[2]),empty=numeric(),single=5)}
 result<-tryCatch(suppressWarnings({
  scale<-switch(c$kind,continuous=scale_size_continuous(limits=fun,transform=c$transform),binned=scale_size_binned(limits=fun,transform=c$transform),identity=scale_alpha_identity(limits=fun,transform=c$transform,guide='legend'))
  mapping<-aes(x,1);mapping[[if(c$kind=='identity')'alpha'else'size']]<-quote(v)
  built<-ggplot_build(ggplot(data.frame(x=seq_along(inputs),v=inputs),mapping)+geom_point()+scale)
  list(values=encode(built$data[[1]][[if(c$kind=='identity')'alpha'else'size']]),keys=unname(lapply(built$plot$guides$params,function(g)list(values=encode(g$key$.value),labels=encode(g$key$.label),mapped=encode(g$key[[if(c$kind=='identity')'alpha'else'size']])))))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(kind=c$kind,transform=c$transform,population=c$population,control=c$control,inputs=c$inputs,seen=seen,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/limit-function-builds.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('captured',length(cases),'primary limit-function builds\n')
