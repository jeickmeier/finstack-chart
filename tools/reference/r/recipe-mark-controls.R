# GG07 independent grid control polygons and actually flattened X-splines.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(grid)
pdf(file=tempfile(fileext='.pdf'),width=7,height=5)
cases=list()
for(curvature in c(-.4,.4,1))for(angle in c(30,60,90,120))for(ncp in c(1,5)) {
 a=c(1,1);b=c(4,3)
 cps=grid:::calcControlPoints(a[1],a[2],b[1],b[2],curvature,angle,ncp)
 x=c(a[1],cps$x,b[1]);y=c(a[2],cps$y,b[2]);
 g=xsplineGrob(x,y,default.units='inches',shape=c(0,rep(.5,ncp),0),open=TRUE)
 pts=xsplinePoints(g)
 cases[[length(cases)+1]]=list(curvature=curvature,angle=angle,ncp=ncp,controls=Map(function(x,y)c(x,y),x,y),points=Map(function(x,y)c(x,y),as.numeric(pts$x),as.numeric(pts$y)))
}
jsonlite::write_json(list(reference='grid R 4.6.1',cases=cases),'fixtures/parity/ggplot2/recipe-mark-controls.json',pretty=TRUE,auto_unbox=TRUE,digits=17)
dev.off()
library(ggplot2)
d=data.frame(x=c(1,1,1,1,2,2),y=c(1,1,1,1,2,2),colour=c(1,1,2,2,1,1),shape=c('a','b','a','a','a','a'),weight=c(1,2,3,-1,0,4))
b=ggplot_build(ggplot(d,aes(x,y,colour=colour,shape=shape,weight=weight))+geom_count())$data[[1]]
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',data=d,result=b),'fixtures/parity/ggplot2/count-mark-controls.json',pretty=TRUE,auto_unbox=TRUE,digits=17,na='null')
d=data.frame(x=1,y=1,v=c(-1,1,-2,2),w=c(1,2,3,4))
b=ggplot_build(ggplot(d,aes(x,y,colour=abs(v),weight=w))+geom_count())$data[[1]]
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',data=d,result=b),'fixtures/parity/ggplot2/count-mark-expression.json',pretty=TRUE,auto_unbox=TRUE,digits=17)
d=data.frame(x=c(1,2),y=c(2,-1),g=c('A','B'))
colcases=list(default=geom_col(),fixed=geom_col(fill='red',colour='blue'),mapped=geom_col(aes(fill=g,colour=g)),mapped_outline=geom_col(aes(colour=g),fill='red'))
colcases=lapply(colcases,function(layer)ggplot_build(ggplot(d,aes(x,y))+layer)$data[[1]])
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=colcases),'fixtures/parity/ggplot2/column-mark-paints.json',pretty=TRUE,auto_unbox=TRUE,digits=17,na='null')
