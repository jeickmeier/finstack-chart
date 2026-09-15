stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile('ggplot-colorbar-',fileext='.pdf'))
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0) 'Infinity' else '-Infinity' else v)
cases=list()
for(nbin in c(0,1,2.5,5)) for(display in c('raster','rectangles','gradient')) for(reverse in c(FALSE,TRUE)) for(direction in c('vertical','horizontal')) for(palette in c('ordinary','asymmetric','discontinuous')) {
 warnings=character()
 result=tryCatch(withCallingHandlers({
   p=ggplot(data.frame(x=c(1,2,3,4),v=c(-2,0,3,8)),aes(x,1,colour=v))+geom_point()+
      scale_colour_gradientn(colours=if(palette=='ordinary')c('#000000','#ffffff')else if(palette=='asymmetric')c('#0000ff','#ffffff','#ff0000')else c('#ff0000','#ff0000','#0000ff','#0000ff'),
       values=if(palette=='ordinary')c(0,1)else if(palette=='asymmetric')c(0,.2,1)else c(0,.499,.5,1),
       limits=c(-2,8),breaks=c(-2,0,3,8),guide=guide_colourbar(nbin=nbin,display=display,reverse=reverse,direction=direction))
   built=ggplot_build(p)
   g=built$plot$guides$params[[1]]
   list(gtable=tryCatch({ggplot_gtable(built);TRUE},error=function(e)list(error=conditionMessage(e))),values=encode(g$key$.value),labels=encode(g$key$.label),colors=encode(g$key$colour),decor_values=encode(g$decor$value),decor_colors=encode(g$decor$colour),mapped=encode(built$data[[1]]$colour),params=g[c('nbin','display','reverse','direction')])
 },warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]=list(nbin=nbin,display=display,reverse=reverse,direction=direction,palette=palette,warnings=as.list(warnings),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/colorbar-boundaries.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
cat(length(cases),'source colorbar geometry records\n')

invisible(dev.off())
