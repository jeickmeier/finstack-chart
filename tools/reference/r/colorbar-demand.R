stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile('ggplot-colorbar-',fileext='.pdf'))
cases=list()
for(n in list(-1,NA_real_,0,1,2.5)) for(population in c('empty','full')) for(selection in c('default','no_breaks','no_labels')) for(hidden in c(FALSE,TRUE)) {
 warnings=character()
 result=tryCatch(withCallingHandlers({
  g=guide_colourbar(nbin=n)
  d=if(population=='full')data.frame(x=1:4,v=c(-2,0,3,8))else data.frame(x=numeric(),v=numeric())
  p=ggplot(d,aes(x,1,colour=v))+geom_point()+scale_colour_gradient(limits=c(-2,8),breaks=if(selection=='no_breaks')NULL else c(-2,0,3,8),labels=if(selection=='no_labels')NULL else waiver(),guide=if(hidden)'none'else g)
  b=ggplot_build(p);ggplot_gtable(b)
  list(guide_count=length(b$plot$guides$params),sample_counts=lapply(b$plot$guides$params,function(p)nrow(p$decor)))
 },warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]=list(nbin=if(is.na(n))'NA'else n,population=population,selection=selection,hidden=hidden,warnings=as.list(warnings),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/colorbar-demand.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
cat(length(cases),'source colourbar demand records\n')

invisible(dev.off())
