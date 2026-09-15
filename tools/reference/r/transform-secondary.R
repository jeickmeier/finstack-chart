stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('scales'))=='1.4.0',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
f<-jsonlite::read_json('fixtures/parity/ggplot2/scale-transform-contracts.json',simplifyVector=FALSE)
encode<-function(v) lapply(unname(v),function(x) if(is.nan(x)) list(number='NaN') else if(is.na(x)) list(number='NA') else if(is.infinite(x)) list(number=if(x>0)'Infinity' else '-Infinity') else x)
records<-list()
pdf(tempfile(fileext='.pdf'))
for(c in f$plots) {
 if(c$route!='position'||c$population!='ordinary')next
 config<-f$cases[[c$configuration+1]];x<-unlist(c$inputs)
 result<-tryCatch(suppressWarnings({
 t<-do.call(getExportedValue('scales',paste0('transform_',config$constructor)),config$args)
 b<-ggplot_build(ggplot(data.frame(x),aes(x,1))+geom_point()+scale_x_continuous(transform=t,n.breaks=3,expand=expansion(0),sec.axis=sec_axis(~2*.x+1)))
 grid::grid.draw(ggplotGrob(b));p<-b$layout$panel_params[[1]]$x.sec$break_info
 list(range=encode(p$range),values=encode(p$major_source_user),positions=encode(p$major),labels=as.list(p$labels))
 }),error=function(e)list(error=conditionMessage(e)))
 records[[length(records)+1L]]<-list(configuration=c$configuration,inputs=c$inputs,result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=records),'fixtures/parity/ggplot2/transform-secondary.json',pretty=TRUE,auto_unbox=TRUE,digits=17,null='null',na='null')
cat(length(records),'actual transform secondary-axis cases captured\n')
