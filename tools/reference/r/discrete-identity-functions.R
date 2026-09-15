# FIX-GG04: primary discrete identity callback demand and ordered domains.
library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
encode <- function(x)lapply(unname(x),function(v)if(is.na(v))NULL else v)
cases <- list()
for(aesthetic in c('colour','fill'))for(population in c('ordinary','nullable','empty','factor'))for(guide in c('legend','none'))for(limit_mode in c('none','reverse','empty','null'))for(break_mode in c('auto','domain','named_reverse')){
 values <- if(aesthetic=='linetype')c('solid','dashed')else c('red','blue')
 unused <- if(aesthetic=='linetype')'dotted'else'green'
 x <- switch(population,ordinary=values,nullable=c(values,NA_character_),empty=character(),factor=factor(values,levels=c(unused,rev(values))))
 limit_calls <- list();break_calls <- list()
 meta <- function(x)list(values=encode(x),is_null=is.null(x),names=as.list(names(x)))
 limits <- function(x){limit_calls[[length(limit_calls)+1L]] <<- meta(x);switch(limit_mode,reverse=rev(x),empty=character(),null=NULL)}
 breaks <- function(x){break_calls[[length(break_calls)+1L]] <<- meta(x);if(break_mode=='domain')x else setNames(rev(x),sprintf('key%d',seq_along(x)))}
 result <- tryCatch(suppressWarnings({
  mapping <- aes(i,1);mapping[[aesthetic]] <- rlang::quo(v)
  scale <- switch(aesthetic,colour=scale_colour_identity,fill=scale_fill_identity,linetype=scale_linetype_identity)(guide=guide,limits=if(limit_mode=='none')NULL else limits,breaks=if(break_mode=='auto')waiver()else breaks,drop=FALSE)
  p <- ggplot(data.frame(i=seq_along(x),v=x),mapping)+geom_point()+scale
  b <- ggplot_build(p);s <- b$plot$scales$get_scales(aesthetic)
  list(mapped=encode(b$data[[1]][[aesthetic]]),trained=encode(s$range$range),keys=unname(lapply(b$plot$guides$params,function(g)list(values=encode(g$key$.value),mapped=encode(g$key[[aesthetic]]),labels=as.list(g$key$.label)))))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]] <- list(aesthetic=aesthetic,population=population,guide=guide,limit_mode=limit_mode,break_mode=break_mode,inputs=encode(x),levels=if(is.factor(x))as.list(levels(x))else NULL,limit_calls=limit_calls,break_calls=break_calls,result=result)
}
paints <- setNames(lapply(c('red','blue','green'),function(v)as.list(setNames(as.integer(col2rgb(v,alpha=TRUE)),c('red','green','blue','alpha')))),c('red','blue','green'))
jsonlite::write_json(list(paints=paints,reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/discrete-identity-functions.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
cat(length(cases),'primary discrete identity callback builds\n')
