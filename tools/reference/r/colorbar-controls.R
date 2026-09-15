stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile('ggplot-colorbar-',fileext='.pdf'))
settings=list(default=list(), samples_zero=list(nbin=0), samples_one=list(nbin=1),samples_two=list(nbin=2), samples_fraction=list(nbin=2.5), samples_negative=list(nbin=-1), samples_na=list(nbin=NA), alpha_zero=list(alpha=0),alpha_half=list(alpha=.5),alpha_negative=list(alpha=-.1),alpha_high=list(alpha=1.1),gradient=list(display='gradient'),rectangles=list(display='rectangles'),display_invalid=list(display='bad'),horizontal=list(direction='horizontal'),direction_invalid=list(direction='bad'),reverse=list(reverse=TRUE),no_limits=list(draw.llim=FALSE,draw.ulim=FALSE),title_hidden=list(title=NULL),title_blank=list(title=''),title_override=list(title='Ramp'),order_negative=list(order=-1),order_fraction=list(order=2.5),order_high=list(order=100),inside=list(position='inside'),top=list(position='top'))
cases=list()
encode <- function(x) lapply(unname(x),function(v) if(is.na(v)) NULL else if(is.infinite(v)) if(v>0) 'Infinity' else '-Infinity' else v)
for(name in names(settings)) {
 warnings=character()
 result=tryCatch(withCallingHandlers({
  guide=do.call(guide_colourbar,settings[[name]])
  p=ggplot(data.frame(x=1:4,v=c(-2,0,3,8)),aes(x,1,colour=v))+geom_point()+scale_colour_gradientn(colours=c('blue','white','red'),values=c(0,.2,1),limits=c(-2,8),breaks=c(-2,0,3,8),guide=guide)
  b=ggplot_build(p);g=b$plot$guides$params[[1]]
  record=list(key=encode(g$key$.value),label=encode(g$key$.label),decor_values=encode(g$decor$value),decor_colors=encode(g$decor$colour),mapped=encode(b$data[[1]]$colour),params=g[c('nbin','display','alpha','reverse','direction','position','title','order','draw_lim')])
  record$gtable=tryCatch({gt=ggplot_gtable(b);list(built=TRUE,names=as.list(gt$layout$name))},error=function(e)list(error=conditionMessage(e)))
  record
 },warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]=list(name=name,warnings=as.list(warnings),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/colorbar-controls.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
cat(length(cases),'source colourbar control/rejection records\n')

invisible(dev.off())
