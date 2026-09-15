stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile('gg05-display-',fileext='.pdf'))
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0) 'Infinity' else '-Infinity' else v)
bar_data <- function(g) {
 result=list(class=as.list(class(g)))
 for(n in c('x','y','width','height')) result[[n]]=encode(as.numeric(g[[n]]))
 fill=g$gp$fill
 if(inherits(fill,'GridLinearGradient')) {
  result$gradient=list(stops=encode(fill$stops),colors=encode(fill$colours),x1=encode(as.numeric(fill$x1)),x2=encode(as.numeric(fill$x2)),y1=encode(as.numeric(fill$y1)),y2=encode(as.numeric(fill$y2)))
 } else result$colors=encode(fill)
 result
}
cases=list()
for(display in c('rectangles','gradient')) for(nbin in c(NA,0,1,2.5,5)) for(constant in c(FALSE,TRUE)) for(direction in c('vertical','horizontal')) for(reverse in c(FALSE,TRUE)) {
 limits=if(constant)c(2,2)else c(-2,8);values=if(constant)rep(2,4)else c(-2,0,3,8)
 args=list(display=display,direction=direction,reverse=reverse);if(!is.na(nbin))args$nbin=nbin
 warnings=character()
 result=tryCatch(withCallingHandlers({
 p=ggplot(data.frame(x=1:4,v=values),aes(x,1,colour=v))+geom_point()+scale_colour_gradientn(colours=c('#0000ff','#ffffff','#ff0000'),values=c(0,.2,1),limits=limits,breaks=unique(values),guide=do.call(guide_colourbar,args))
 b=ggplot_build(p);params=b$plot$guides$params[[1]];gt=ggplot_gtable(b);box=gt$grobs[[which(gt$layout$name=='guide-box-right')]];guide=box$grobs[[1]]
 list(mapped=encode(b$data[[1]]$colour),decor_colors=encode(params$decor$colour),keys=encode(params$key$.value),labels=encode(params$key$.label),decor_values=encode(params$decor$value),nbin=params$nbin,bar=bar_data(guide$grobs[[which(guide$layout$name=='bar')]]))
 },warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]=list(palette='asymmetric',display=display,constant=constant,nbin=if(is.na(nbin))NULL else nbin,direction=direction,reverse=reverse,warnings=as.list(warnings),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/colorbar-display.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
invisible(dev.off())
cat(length(cases),'colorbar display cases\n')
