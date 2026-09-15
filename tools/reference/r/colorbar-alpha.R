stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile('gg05-alpha-',fileext='.pdf'))
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0) 'Infinity' else '-Infinity' else v)
cases=list()
for(display in c('raster','rectangles','gradient')) for(palette in c('asymmetric','transparent')) for(alpha in c(NA,0,.5,1)) for(direction in c('vertical','horizontal')) for(reverse in c(FALSE,TRUE)) {
 warnings=character()
 result=tryCatch(withCallingHandlers({
 colors=if(palette=='asymmetric')c('#0000FF','#FFFFFF','#FF0000')else c('#0000FF20','#FFFFFF80','#FF0000E0')
 p=ggplot(data.frame(x=1:4,v=c(-2,0,3,8)),aes(x,1,colour=v))+geom_point()+scale_colour_gradientn(colours=colors,values=c(0,.2,1),limits=c(-2,8),breaks=c(-2,0,3,8),guide=guide_colourbar(nbin=5,display=display,alpha=alpha,direction=direction,reverse=reverse))
 b=ggplot_build(p);params=b$plot$guides$params[[1]];gt=ggplot_gtable(b)
 list(mapped=encode(b$data[[1]]$colour),decor_colors=encode(params$decor$colour),keys=encode(params$key$.value),labels=encode(params$key$.label),decor_values=encode(params$decor$value),built=TRUE)
 },warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]=list(palette=palette,display=display,alpha=if(is.na(alpha))NULL else alpha,nbin=5,direction=direction,reverse=reverse,warnings=as.list(warnings),result=result)
}
rejections=list()
for(alpha in c(-.1,1.1,Inf,-Inf,NaN)) for(hidden in c(FALSE,TRUE)) {
 result=tryCatch({
 guide=guide_colourbar(alpha=alpha,nbin=5);if(hidden)guide='none'
 p=ggplot(data.frame(x=1:4,v=c(-2,0,3,8)),aes(x,1,colour=v))+geom_point()+scale_colour_gradient(low='blue',high='red',guide=guide)
 b=ggplot_build(p);ggplot_gtable(b)
 list(accepted=TRUE,decor_colors=if(hidden)list()else encode(b$plot$guides$params[[1]]$decor$colour))
 },error=function(e)list(error=conditionMessage(e)))
 rejections[[length(rejections)+1L]]=list(alpha=if(is.nan(alpha))'NaN'else if(is.infinite(alpha))if(alpha>0)'Infinity'else'-Infinity'else alpha,hidden=hidden,result=result)
}
missing=list()
for(alpha in c(NA,0,.5,1,Inf,-Inf)) missing[[length(missing)+1L]]=list(alpha=if(is.na(alpha))NULL else if(is.infinite(alpha))if(alpha>0)'Infinity'else'-Infinity'else alpha,colors=encode(scales::alpha(c(NA_character_,'#00000000','grey50'),alpha)))
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases,constructor_boundaries=rejections,missing=missing),'fixtures/parity/ggplot2/colorbar-alpha.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
invisible(dev.off())
cat(length(cases),'colorbar alpha cases and',length(rejections),'constructor boundaries\n')
