# FIX-GG04: identity vector limit and break callbacks, including hidden-guide timing.
library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
encode <- function(x) lapply(unname(x),function(v) if(is.nan(v))list(number='NaN')else if(is.na(v))list(number='NA')else if(is.infinite(v))list(number=if(v>0)'Infinity'else'-Infinity')else v)
cases <- list();configs <- list()
for(family in c('cardinality','center'))for(composed in c(FALSE,TRUE)) {
 configs[[length(configs)+1L]] <- list(family=family,composed=composed)
 forward <- function(x)if(family=='cardinality')x+length(x)else x-mean(x,na.rm=TRUE)
 inverse <- function(x)if(family=='cardinality')x-length(x)else x
 tr <- scales::new_transform(family,forward,inverse)
 if(composed)tr <- tryCatch(scales::transform_compose(tr,scales::transform_reverse()),error=function(e)list(error=conditionMessage(e)))
 for(population in c('ordinary','nullable','empty'))for(guide in c('legend','none'))for(layers in c('one','two'))for(limit_mode in c('none','reverse'))for(break_mode in c('auto','domain','mixed')) {
  x <- switch(population,ordinary=c(1,2,4),nullable=c(1,NA_real_,4),empty=numeric())
  y <- if(population=='empty')numeric()else c(2,10)
  limit_calls <- list();break_calls <- list()
  limit_fn <- function(x){limit_calls[[length(limit_calls)+1L]] <<- list(values=encode(x),names=as.list(names(x)),is_null=is.null(x));rev(x)}
  break_fn <- function(x){break_calls[[length(break_calls)+1L]] <<- list(values=encode(x),names=as.list(names(x)),is_null=is.null(x));if(break_mode=='domain')x else setNames(c(x[2],mean(x),x[1],x[1],NA,Inf,-Inf),c('last','middle','first','again','missing','positive','negative'))}
  result <- if(!is.null(tr$error))tr else tryCatch(suppressWarnings({
   p <- ggplot(data.frame(i=seq_along(x),v=x),aes(i,1,size=v))+geom_point()+scale_size_identity(transform=tr,guide=guide,limits=if(limit_mode=='none')NULL else limit_fn,breaks=if(break_mode=='auto')waiver()else break_fn)
   if(layers=='two')p <- p+geom_point(data=data.frame(i=seq_along(y),v=y))
   b <- ggplot_build(p);s <- b$plot$scales$get_scales('size')
   list(mapped=lapply(b$data,function(d)encode(d$size)),trained=encode(s$range$range),keys=unname(lapply(b$plot$guides$params,function(g)list(values=encode(g$key$.value),mapped=encode(g$key$size),labels=as.list(g$key$.label)))))
  }),error=function(e)list(error=conditionMessage(e)))
  cases[[length(cases)+1L]] <- list(configuration=length(configs)-1L,population=population,guide=guide,layers=layers,limit_mode=limit_mode,break_mode=break_mode,inputs=encode(x),second=encode(y),limit_calls=limit_calls,break_calls=break_calls,result=result)
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',configurations=configs,cases=cases),'fixtures/parity/ggplot2/identity-vector-functions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat(length(cases),'identity vector callback builds captured\n')
