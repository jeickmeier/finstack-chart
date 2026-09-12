# FIX-GG04 / GG2-03: the positional palette maps category indices before range training.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(unname(v), function(x) if(is.na(x)) NULL else if(is.infinite(x)) if(x>0) 'Infinity' else '-Infinity' else x)
populations <- list(mixed=c('b',NA,'NA','a'),finite=c('b','NA','a'),missing=c(NA_character_,NA_character_),empty=character())
limits <- list(auto=NULL, authored=c(NA,'b','NA','a'))
palettes <- list(default=function(n) seq_len(n), reverse=function(n) rev(seq_len(n)),
 spread=function(n) seq_len(n)*2, shifted=function(n) seq_len(n)+4, fractional=function(n) seq_len(n)/2,
 repeated=function(n) rep(2,n), short=function(n) rep(2,max(0,n-1)), empty=function(n) numeric(),
 nonfinite=function(n) rep_len(c(NA_real_,Inf,-Inf,2),n))
cases<-list()
for(population in names(populations)) for(limit_name in names(limits)) for(translate in c(FALSE,TRUE)) for(palette_name in names(palettes)) {
 values<-populations[[population]]; palette<-palettes[[palette_name]]
 training_scale<-scale_x_discrete(limits=limits[[limit_name]],na.translate=translate); training_scale$train(values)
 palette_values<-palette(if(training_scale$is_empty()) 0 else length(training_scale$get_limits()))
 result<-tryCatch(suppressWarnings({
  p<-ggplot(data.frame(x=values,y=seq_along(values)),aes(x,y))+geom_point()+
   scale_x_discrete(limits=limits[[limit_name]],na.translate=translate,palette=palette)
  b<-ggplot_build(p);panel<-b$layout$panel_params[[1]];scale<-panel$x$scale
  secondary <- tryCatch(suppressMessages(suppressWarnings({
   b2<-ggplot_build(p+scale_x_discrete(limits=limits[[limit_name]],na.translate=translate,palette=palette,sec.axis=dup_axis()))
   s2<-b2$layout$panel_params[[1]]$x.sec
   list(breaks=encode(s2$get_breaks()),labels=as.list(unname(s2$get_labels())),positions=encode(s2$break_positions()))
  })),error=function(e)list(error=conditionMessage(e)))
  list(limits=as.list(unname(scale$get_limits())), palette=encode(palette(if(scale$is_empty()) 0 else length(scale$get_limits()))),
   range=encode(panel$x$continuous_range), mapped=encode(b$data[[1]]$x), positions=encode(panel$x$rescale(b$data[[1]]$x)),
   breaks=as.list(unname(panel$x$get_breaks())),labels=as.list(unname(panel$x$get_labels())),major_positions=encode(panel$x$break_positions()),
   secondary=secondary)
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(population=population,inputs=as.list(values),limits_name=limit_name,limits=if(is.null(limits[[limit_name]])) NULL else as.list(limits[[limit_name]]),na_translate=translate,palette_name=palette_name,palette_values=encode(palette_values),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/discrete-position-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(cases),'discrete positional palette panels\n')
