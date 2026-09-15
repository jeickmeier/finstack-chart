# FIX-GG04: identity transformation preserves source vector batches.
library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
encode <- function(x) lapply(unname(x),function(v) if(is.nan(v))list(number='NaN')else if(is.na(v))list(number='NA')else if(is.infinite(v))list(number=if(v>0)'Infinity'else'-Infinity')else v)
cases <- list();configs <- list()
for(family in c('cardinality','center'))for(composed in c(FALSE,TRUE)) {
 configs[[length(configs)+1L]] <- list(family=family,composed=composed)
 forward <- function(x)if(family=='cardinality')x+length(x)else x-mean(x,na.rm=TRUE)
 inverse <- function(x)if(family=='cardinality')x-length(x)else x
 t <- scales::new_transform(family,forward,inverse)
 if(composed)t <- tryCatch(scales::transform_compose(t,scales::transform_reverse()),error=function(e)list(error=conditionMessage(e)))
 for(population in c('ordinary','nullable','empty'))for(guide in c('legend','none'))for(limits in c('none','fixed'))for(layers in c('one','two')) {
  x <- switch(population,ordinary=c(1,2,4),nullable=c(1,NA_real_,4),empty=numeric())
  y <- if(population=='empty')numeric()else c(2,10)
  result <- if(!is.null(t$error))t else tryCatch(suppressWarnings({
   p <- ggplot(data.frame(i=seq_along(x),v=x),aes(i,1,size=v))+geom_point()+scale_size_identity(transform=t,guide=guide,limits=if(limits=='fixed')c(0,10)else NULL)
   if(layers=='two')p <- p+geom_point(data=data.frame(i=seq_along(y),v=y))
   b <- ggplot_build(p);s <- b$plot$scales$get_scales('size')
   list(mapped=lapply(b$data,function(d)encode(d$size)),limits=encode(s$get_limits()),keys=unname(lapply(b$plot$guides$params,function(g)list(values=encode(g$key$.value),mapped=encode(g$key$size),labels=as.list(g$key$.label)))))
  }),error=function(e)list(error=conditionMessage(e)))
  cases[[length(cases)+1L]] <- list(configuration=length(configs)-1L,population=population,guide=guide,limits=limits,layers=layers,inputs=encode(x),second=encode(y),result=result)
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',configurations=configs,cases=cases),'fixtures/parity/ggplot2/identity-vector-transforms.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat(length(cases),'identity vector transform builds captured\n')
