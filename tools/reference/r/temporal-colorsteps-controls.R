pdf(file = tempfile(fileext = ".pdf"))
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile(fileext='.pdf'))
enc=function(x)lapply(unname(x),function(v)if(is.na(v))NULL else v)
cases=list()
for(kind in c('date','datetime'))for(even in c(TRUE,FALSE))for(show in c(FALSE,TRUE))for(mode in c('auto','explicit')) {
 time=function(x)if(kind=='date')as.Date(x,origin='2020-01-01')else as.POSIXct(x*3600,origin='2020-01-01',tz='UTC')
 args=list(limits=time(c(0,10)),guide=guide_coloursteps(even.steps=even,show.limits=show))
 if(mode=='explicit')args$breaks=time(c(-1,0,1,1,3,20,NA_real_))
 scale=do.call(get(paste0('scale_colour_',kind)),args)
 b=ggplot_build(ggplot(data.frame(x=1:4,v=time(c(0,2,8,10))),aes(x,1,colour=v))+geom_point()+scale)
 g=ggplot_gtable(b);p=b$plot$guides$params[[1]]
 cases[[length(cases)+1]]=list(kind=kind,channel='colour',limits='full',population='ordinary',guide='coloursteps',breaks=mode,inputs=as.list(c(0,2,8,10)),even_steps=even,show_limits=show,key=lapply(p$key,enc),decor=lapply(p$decor,enc))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/temporal-colorsteps-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
invisible(dev.off())
