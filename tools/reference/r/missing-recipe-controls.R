# Development-only pinned missing-versus-absent recipe control oracle.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
d=data.frame(x=c(1,2),y=c(1,2),w=c(NA,1),a=c(NA,0),r=c(1,NA))
out=list()
for(name in c('column','tile','errorbar','spoke','reference')){
 g=switch(name,column=geom_col(aes(width=w)),tile=geom_tile(aes(width=w)),errorbar=geom_errorbar(aes(width=w,ymin=y-.5,ymax=y+.5)),spoke=geom_spoke(aes(angle=a,radius=r)),reference=geom_abline(aes(slope=1,intercept=w)))
 b=ggplot_build(ggplot(d,aes(x,y))+g)$data[[1]]
 out[[name]]=b[,intersect(names(b),c('x','y','xmin','xmax','xend','yend','angle','radius','slope','intercept'))]
}
jsonlite::write_json(list(reference=list(R=as.character(getRversion()),ggplot2=as.character(packageVersion('ggplot2'))),data=d,values=out),'fixtures/parity/ggplot2/missing-recipe-controls.json',auto_unbox=TRUE,na='null',pretty=TRUE,digits=17)
