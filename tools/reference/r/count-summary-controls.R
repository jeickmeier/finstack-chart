# FIX-GG06 source populations plus independent R stats helper anchors.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
pdf(file=tempfile(fileext='.pdf'))
inputs=list(c(1,2,4,8),c(-10,0,1,3,20),c(7),rep(2,5),c(1e8,1e8+1,1e8+3))
helpers=list()
for(x in inputs) {
 for(mult in c(1,2,-1)) {
  helpers[[length(helpers)+1]]=list(x=I(x),helper=list(MeanSe=list(mult=mult)),expected=unname(as.numeric(mean_se(x,mult))))
  helpers[[length(helpers)+1]]=list(x=I(x),helper=list(MeanSdl=list(mult=mult)),expected=c(mean(x),mean(x)-mult*sd(x),mean(x)+mult*sd(x)))
 }
 for(p in c(.5,.95,.99)) {
  helpers[[length(helpers)+1]]=list(x=I(x),helper=list(MedianHilow=list(confidence=p)),expected=unname(quantile(x,c(.5,(1-p)/2,(1+p)/2))))
  h=qt((1+p)/2,length(x)-1)*sd(x)/sqrt(length(x))
  helpers[[length(helpers)+1]]=list(x=I(x),helper=list(MeanClNormal=list(confidence=p)),expected=c(mean(x),mean(x)-h,mean(x)+h))
 }
 n=length(x); draws=lapply(0:7,function(k)I(as.integer((seq_len(n)*k+k)%%n)))
 boot=vapply(draws,function(d)mean(x[d+1]),numeric(1))
 helpers[[length(helpers)+1]]=list(x=I(x),helper=list(MeanClBoot=list(confidence=.95,draws=draws)),expected=c(mean(x),unname(quantile(boot,c(.025,.975)))))
}
x=c(1,2,4,8)
draws=list(c(3,0,3,0),c(3,2,1,0)); boot=vapply(draws,function(d)mean(x[d+1]),numeric(1))
helpers[[length(helpers)+1]]=list(x=I(x),helper=list(MeanClBootSeeded=list(confidence=.95,samples=2,seed='0')),expected=c(mean(x),unname(quantile(boot,c(.025,.975)))))
d=data.frame(x=c(1,1,2,2,4,4),y=c(1,3,4,8,2,6),weight=c(2,-1,0,3,NA,-2),group=c(1,1,1,1,2,2))
cnt=ggplot_build(ggplot(d,aes(x,weight=weight,group=group))+stat_count())$data[[1]]
summ=ggplot_build(ggplot(d,aes(x,y,group=group,weight=weight))+stat_summary())$data[[1]]
bins=list()
for(cl in c('right','left')) {
 z=ggplot_build(ggplot(d,aes(x,y,group=group))+stat_summary_bin(breaks=c(0,2,5),closed=cl,fun.data=mean_se))$data[[1]]
 bins[[cl]]=z[,intersect(c('x','y','ymin','ymax','width','group'),names(z)),drop=FALSE]
}
extra=list()
for(mode in c('automatic','transformed','faceted','free')) {
 p=ggplot(d,aes(x,y,group=group))+stat_summary_bin(bins=3,fun.data=mean_se)
 if(mode=='transformed')p=p+scale_y_log10()
 if(mode=='faceted')p=p+facet_wrap(~group)
 if(mode=='free')p=p+facet_wrap(~group,scales='free_x')
 z=ggplot_build(p)$data[[1]]
 extra[[mode]]=z[,intersect(c('x','y','ymin','ymax','width','group','PANEL'),names(z)),drop=FALSE]
}
h=ggplot_build(ggplot(d,aes(x,group=group))+stat_bin(bins=3)+facet_wrap(~group))$data[[1]]
extra$histogram=h[,c('x','xmin','xmax','count','group','PANEL')]
overlay=data.frame(x=c(0,10),y=c(1,1))
z=ggplot_build(ggplot(data.frame(x=c(1,2)),aes(x))+stat_bin(bins=3)+geom_point(data=overlay,aes(x,y)))$data[[1]]
extra$overlay=z[,c('x','xmin','xmax','count')]
z=ggplot_build(ggplot(data.frame(x=c(1,2,10)),aes(x))+stat_bin(bins=3)+scale_x_continuous(limits=c(0,5),oob=scales::oob_keep))$data[[1]]
extra$kept_limits=z[,c('x','xmin','xmax','count')]
hd=data.frame(y=c(0,1,100,101),g=c('A','A','B','B'))
for(sc in c('fixed','free_x','free_y')) {
 z=ggplot_build(ggplot(hd,aes(y=y))+stat_bin(bins=3,orientation='y')+facet_wrap(~g,scales=sc))$data[[1]]
 extra[[paste0('horizontal_',sc)]]=z[,c('y','ymin','ymax','count','PANEL')]
}
z=ggplot_build(ggplot(data.frame(y=c(1,2)),aes(y=y))+stat_bin(bins=3,orientation='y')+geom_point(data=data.frame(x=c(1,1),y=c(0,10)),aes(x,y)))$data[[1]]
extra$horizontal_overlay=z[,c('y','ymin','ymax','count')]
jsonlite::write_json(list(reference='R 4.6.1 / ggplot2 4.0.3; mean_sdl/hilow/normal/bootstrap independently evaluated from R stats formulas; Hmisc runtime unavailable',helpers=helpers,extra=extra,data=d,count=cnt[,c('x','count','prop','width','group')],summary=summ[,c('x','y','ymin','ymax','group')],bins=bins),'fixtures/parity/ggplot2/count-summary-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null')
