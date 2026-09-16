# GG12 free-grid scale callback sharing and margin occurrence reference.
library(ggplot2);library(grid)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
pdf(file=tempfile(fileext='.pdf'),width=8,height=6)
d=data.frame(x=c(1,2,3,4),y=c(2,4,6,8),r=c('A','A','B','B'),c=c('L','R','L','R'));cases=list()
for(margins in c(FALSE,TRUE))for(mode in c('identity','reverse','scalar')){
 calls=list();callback=function(x,range){calls[[length(calls)+1L]]<<-list(values=x,range=range);switch(mode,identity=x,reverse=rev(x),scalar=mean(x))}
 p=ggplot(d,aes(x,y))+geom_point()+scale_y_continuous(oob=callback)+facet_grid(r~c,scales='free_y',margins=margins)
 entry=list(margins=margins,mode=mode)
 entry$result=tryCatch({b=ggplot_build(p);g=ggplot_gtable(b);grid.newpage();grid.draw(g);list(ok=TRUE,data=b$data,layout=b$layout$layout)},error=function(e)list(ok=FALSE,error=conditionMessage(e)))
 entry$calls=calls;cases[[length(cases)+1L]]=entry
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/facet-vector-sharing.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null',null='null')
dev.off();cat('Captured',length(cases),'GG12 vector-sharing cases\n')
