# GG-04: composed transform arithmetic, domains and actual scale consumers.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3', as.character(packageVersion('scales')) == '1.4.0')
library(ggplot2)
encode <- function(v) lapply(unname(v), function(x) {
 if(is.nan(x)) return(list(number='NaN'))
 if(is.na(x)) return(list(number='NA'))
 if(is.infinite(x)) return(list(number=if(x>0) 'Infinity' else '-Infinity'))
 x
})
leaf <- function(name, ...) list(name=name, args=list(...))
configs <- list(list(), list(leaf('identity')), list(leaf('reverse')), list(leaf('log10'),leaf('reverse')),
 list(leaf('reverse'),leaf('reverse')), list(leaf('sqrt'),leaf('log10')),
 list(leaf('log1p'),leaf('sqrt')), list(leaf('asinh'),leaf('reverse')),
 list(leaf('boxcox',p=.5,offset=2),leaf('reverse')),
 list(leaf('reverse'),leaf('log10')), list(leaf('reciprocal'),leaf('sqrt')),
 list(leaf('modulus',p=.5),leaf('asinh')), list(leaf('log10'),leaf('sqrt'),leaf('reverse')))
make <- function(config) do.call(scales::transform_compose,lapply(config,function(x)do.call(getExportedValue('scales',paste0('transform_',x$name)),x$args)))
cases <- list(); plots <- list();pdf(tempfile(fileext='.pdf'))
for(i in seq_along(configs)) {
 config <- configs[[i]]; inputs <- c(-Inf,-4,-1,0,.01,.25,.5,1,2,4,10,Inf,NA_real_,NaN)
 evaluated <- tryCatch(suppressWarnings({
  trans <- make(config)
  evaluate <- function(f) tryCatch(list(values=encode(f(inputs))),error=function(e)list(error=conditionMessage(e)))
  list(domain=encode(trans$domain),forward=evaluate(trans$transform),inverse=evaluate(trans$inverse))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[i]] <- list(transforms=config,inputs=encode(inputs),result=evaluated)
 for(route in c('position','paint','binned_paint')) for(population in c('positive','mixed','nonfinite')) for(count in if(route=='position') c(NA_real_,3,7) else NA_real_) {
  values <- switch(population,positive=c(.01,.25,.5,1,2,4),mixed=c(-4,-1,0,.25,1,4),nonfinite=c(.01,.25,1,4,NA_real_,-Inf,Inf))
  result <- tryCatch(suppressWarnings({
   trans <- make(config);d <- data.frame(value=values,i=seq_along(values))
   p <- if(route=='position') ggplot(d,aes(value,1))+geom_point()+scale_x_continuous(transform=trans,n.breaks=if(is.na(count))NULL else count) else
        ggplot(d,aes(i,1,colour=value))+geom_point() + if(route=='paint') scale_colour_gradient(transform=trans,guide='none') else scale_colour_steps(transform=trans,guide='none')
   b <- ggplot_build(p);grid::grid.draw(ggplotGrob(b));panel <- b$layout$panel_params[[1]]$x
   list(mapped=if(route=='position')encode(b$data[[1]]$x) else as.list(b$data[[1]]$colour),range=encode(panel$continuous_range),breaks=encode(panel$get_breaks()),positions=encode(panel$break_positions()),labels=as.list(panel$get_labels()))
  }),error=function(e)list(error=conditionMessage(e)))
  plots[[length(plots)+1L]] <- list(configuration=i-1L,route=route,population=population,count=if(is.na(count))NULL else count,inputs=encode(values),result=result)
 }
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1 / scales 1.4.0',cases=cases,plots=plots),'fixtures/parity/ggplot2/transform-compositions.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('Captured',length(cases),'compositions and',length(plots),'actual plots; reference only\n')
