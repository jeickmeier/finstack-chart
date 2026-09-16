pdf(file=tempfile(fileext='.pdf'))
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases=list()
for(kind in c('dodge','dodge2')) for(preserve in c('total','single')) for(reverse in c(FALSE,TRUE)) for(variable in c(FALSE,TRUE)) {
 d=data.frame(x=c(1,1,2),xmin=c(.6,.6,1.6),xmax=c(1.4,1.4,2.4),y=c(1,2,3),group=c(1L,2L,1L),PANEL=1L)
 if(variable){d$xmin[2]=.8;d$xmax[2]=1.2}
 p=if(kind=='dodge')position_dodge(.8,preserve=preserve,reverse=reverse) else position_dodge2(.8,preserve=preserve,reverse=reverse,padding=.1)
 params=p$setup_params(d); out=p$compute_panel(p$setup_data(d,params),params,list());out=out[order(out$y),]
 cases[[length(cases)+1]]=list(kind=kind,preserve=preserve,reverse=reverse,variable=variable,input=d[c('x','xmin','xmax','y','group')],output=out[c('xmin','xmax')])
}
for(fill in c(FALSE,TRUE)) for(reverse in c(FALSE,TRUE)) for(vjust in c(0,.5,1)) {
 d=data.frame(x=1,y=c(2,3,-1,-4,0),group=1:5,PANEL=1L)
 p=if(fill)position_fill(vjust=vjust,reverse=reverse) else position_stack(vjust=vjust,reverse=reverse)
 params=p$setup_params(d);out=p$compute_panel(p$setup_data(d,params),params,list());out=out[order(out$group),]
 cases[[length(cases)+1]]=list(kind='stack',fill=fill,reverse=reverse,vjust=vjust,input=d[c('x','y','group')],output=out[c('y','ymin','ymax')])
}
for(kind in c('dodge','dodge2')) {
 d=data.frame(x=c(1,1,1),xmin=.6,xmax=1.4,y=1:3,group=c(1L,2L,1L),PANEL=c(1L,1L,2L))
 p=if(kind=='dodge')position_dodge(.8,preserve='single') else position_dodge2(.8,preserve='single')
 params=p$setup_params(d); out=do.call(rbind,lapply(split(d,d$PANEL),function(panel)p$compute_panel(p$setup_data(panel,params),params,list())[c("xmin","xmax","y")]));out=out[order(out$y),]
 cases[[length(cases)+1]]=list(kind=kind,preserve='single',reverse=FALSE,panels=TRUE,count=params$n,input=d[c('x','xmin','xmax','y','group','PANEL')],output=out[c('xmin','xmax')])
}
for(fill in c(FALSE,TRUE)) {
 d=data.frame(x=1:2,y=c(-1,2),group=1:2,PANEL=1L)
 p=if(fill)position_fill() else position_stack()
 params=p$setup_params(d);out=p$compute_panel(p$setup_data(d,params),params,list())
 cases[[length(cases)+1]]=list(kind='stack',fill=fill,reverse=FALSE,vjust=1,input=d[c('x','y','group')],output=out['y'])
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/position-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17)
