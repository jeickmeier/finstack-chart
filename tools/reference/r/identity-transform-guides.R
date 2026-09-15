stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('scales'))=='1.4.0',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
f <- jsonlite::read_json('fixtures/parity/ggplot2/scale-transform-contracts.json',simplifyVector=FALSE)
encode <- function(v) lapply(unname(v),function(x) if(is.nan(x))list(number='NaN')else if(is.na(x))list(number='NA')else if(is.infinite(x))list(number=if(x>0)'Infinity'else'-Infinity')else x)
records <- list()
for(c in f$plots) {
 if(c$route!='position'||c$population!='ordinary')next
 config <- f$cases[[c$configuration+1]]
 for(pop in c('ordinary','empty')) for(guide in c('legend','none')) for(mode in c('auto','explicit')) {
  x <- if(pop=='empty')numeric()else unlist(c$inputs)
  result <- tryCatch(suppressWarnings({
   t <- do.call(getExportedValue('scales',paste0('transform_',config$constructor)),config$args)
   b <- ggplot_build(ggplot(data.frame(x=seq_along(x),v=x),aes(x,x,size=v))+geom_point()+scale_size_identity(transform=t,guide=guide,breaks=if(mode=='auto')waiver()else x))
   s <- b$plot$scales$get_scales('size')
   keys <- lapply(b$plot$guides$params,function(g)list(values=encode(g$key$.value),mapped=encode(g$key$size),labels=as.list(g$key$.label)))
   list(mapped=encode(b$data[[1]]$size),limits=encode(s$get_limits()),keys=unname(keys))
  }),error=function(e)list(error=conditionMessage(e)))
  records[[length(records)+1L]] <- list(configuration=c$configuration,population=pop,guide=guide,break_mode=mode,inputs=encode(x),result=result)
 }
}
jsonlite::write_json(list(reference=f$reference,configurations=f$cases,cases=records),'fixtures/parity/ggplot2/identity-transform-guides.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat(length(records),'identity transform guide builds captured\n')
