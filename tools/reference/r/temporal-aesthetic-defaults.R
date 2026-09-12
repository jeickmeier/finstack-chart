# FIX-GG04 / GG2-03: inferred Date/datetime nonpositional scale defaults.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity' else '-Infinity' else x)
sets<-list(spaced=c(0,1,4,9),constant=c(4,4,4),missing=c(NA,0,4,NA,9),all_missing=c(NA_real_,NA_real_),empty=numeric(),unsorted=c(9,0,4,1))
cases<-list()
for(kind in c('date','datetime'))for(population in names(sets))for(channel in c('size','alpha','linewidth','colour','fill')){
 raw<-sets[[population]];when<-if(kind=='date')as.Date(raw+19723,origin='1970-01-01')else as.POSIXct(raw+1704067200,origin='1970-01-01',tz='UTC')
 result<-tryCatch(suppressWarnings({
  mapping<-aes(x=seq_along(when),y=seq_along(when));mapping[[channel]]<-rlang::new_quosure(quote(when))
  p<-ggplot(data.frame(when=when),mapping)+(if(channel=='linewidth')geom_line()else geom_point(shape=21))
  b<-ggplot_build(p);s<-b$plot$scales$get_scales(channel)
  guide<-if(is.null(s))NULL else tryCatch(list(breaks=encode(s$get_breaks()),labels=as.list(unname(s$get_labels()))),error=function(e)list(error=conditionMessage(e)))
  list(values=encode(b$data[[1]][[channel]]), scale_installed=!is.null(s), limits=if(is.null(s))NULL else encode(s$get_limits()),guide=guide, point_count=if(channel=='linewidth')NULL else sum(!is.na(b$data[[1]]$size)))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,population=population,channel=channel,inputs=encode(as.numeric(when)),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/temporal-aesthetic-defaults.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'temporal aesthetic reference builds\n')
