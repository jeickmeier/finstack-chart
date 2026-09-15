stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile('gg05-ticks-',fileext='.pdf'))
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0) 'Infinity' else '-Infinity' else v)
walk <- function(g) {
 result=list(class=as.list(class(g)))
 for(n in c('x','y','x0','x1','y0','y1','label')) if(!is.null(g[[n]])) result[[n]]=if(grid::is.unit(g[[n]]))list(value=encode(as.numeric(g[[n]])),unit=as.list(grid::unitType(g[[n]]))) else encode(g[[n]])
 if(!is.null(g$children)) result$children=unname(lapply(g$children,walk))
 result
}
ticks <- function(g, direction) {
 if(inherits(g,'polyline')) {
  x=as.numeric(g[[if(direction=='horizontal')'x' else 'y']])
  return(x[seq.int(1L,length(x),by=2L)])
 }
 for(child in g$children) {
  result=ticks(child,direction)
  if(length(result)) return(result)
 }
 numeric()
}
cases=list()
for(labels in c('automatic','hidden')) for(lower in c(TRUE,FALSE)) for(upper in c(TRUE,FALSE)) for(nbin in c(0,1,5,1e-320)) for(direction in c('vertical','horizontal')) for(reverse in c(FALSE,TRUE)) {
 p=ggplot(data.frame(x=1:4,v=c(-2,0,3,8)),aes(x,1,colour=v))+geom_point()+scale_colour_gradient(low='#000000',high='#FFFFFF',limits=c(-2,8),breaks=c(-2,0,3,8),labels=if(labels=='hidden')NULL else waiver(),guide=guide_colourbar(nbin=nbin,direction=direction,reverse=reverse,draw.llim=lower,draw.ulim=upper))
 b=ggplot_build(p);params=b$plot$guides$params[[1]];gt=ggplot_gtable(b);box=gt$grobs[[which(gt$layout$name=='guide-box-right')]];guide=box$grobs[[1]]
 result=list(mapped=encode(b$data[[1]]$colour),tick_positions=encode(ticks(guide$grobs[[which(guide$layout$name=='ticks')]],direction)),decor_colors=encode(params$decor$colour),keys=encode(params$key$.value),labels=encode(params$key$.label),decor_values=encode(params$decor$value),grobs=setNames(lapply(guide$grobs,walk),guide$layout$name))
 cases[[length(cases)+1L]]=list(palette='ordinary',display='raster',labels=labels,lower=lower,upper=upper,nbin=nbin,direction=direction,reverse=reverse,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/colorbar-presentation.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
invisible(dev.off())
cat(length(cases),'raster presentation cases\n')
