# GG09 development oracle only; a captured unsupported dependency is not acceptance.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases=list()
capture=function(name,controls,fn){warnings=character();messages=character();result=withCallingHandlers(tryCatch(list(ok=TRUE,value=fn()),error=function(e)list(ok=FALSE,error=conditionMessage(e))),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')},message=function(m){messages<<-c(messages,conditionMessage(m));invokeRestart('muffleMessage')});cases[[length(cases)+1]]<<-list(name=name,controls=controls,result=result,warnings=warnings,messages=messages)}
built=function(p){v=ggplot_build(p)$data[[1]];v[intersect(names(v),c("x","y","PANEL","group","ymin","lower","middle","upper","ymax","outliers","notchlower","notchupper","relvarwidth","width","density","scaled","ndensity","count","wdensity","n","violinwidth","quantile","binwidth","ncount","stackpos","xmin","xmax"))]}
sets=list(tied=c(0,0,1,2,3,3,20),constant=rep(2,4),singleton=2,even=c(-2,0,4,8),missing=c(NA,-Inf,1,2,Inf))
for(name in names(sets))for(type in 1:9) {
 y=sets[[name]];capture(paste0('box-',name,'-',type),list(family='boxplot',y=y,quantile_type=type),function()built(ggplot(data.frame(x=1,y=y),aes(x,y))+geom_boxplot(quantile.type=type)))
}
for(weights in list(c(1,1,1,1),c(0,1,3,2),c(1,-1,2,3))) {
 capture(paste0('box-weighted-',length(cases)),list(family='boxplot',y=c(0,1,4,10),weight=weights),function()built(ggplot(data.frame(x=1,y=c(0,1,4,10),w=weights),aes(x,y,weight=w))+geom_boxplot()))
}
x=c(-1,-.7,-.5,-.2,0,.1,.2,.3,.4,.7,1,1.2,1.4,1.8,2,2.1,2.4,2.8,3,4)
for(kernel in c('gaussian','epanechnikov','rectangular','triangular','biweight','cosine','optcosine'))for(weighted in c(FALSE,TRUE)) {
 w=if(weighted)seq_along(x)else NULL
 capture(paste('density',kernel,weighted,sep='-'),list(family='density',x=x,weight=w,bw=.45,adjust=1,kernel=kernel,n=32,from=-2,to=5),function()ggplot2:::compute_density(x,w,-2,5,bw=.45,kernel=kernel,n=32))
}
for(bw in c('nrd0','nrd','ucv','bcv','SJ-ste','SJ-dpi')) {
 capture(paste0('bandwidth-',bw),list(family='density',x=x,bw=bw,n=32,from=-2,to=5),function()list(bw=ggplot2:::calc_bw(x,bw),values=ggplot2:::compute_density(x,NULL,-2,5,bw=bw,n=32)))
}
for(bounds in list(c(0,Inf),c(0,3),c(3,0))) {
 capture(paste0('density-bounds-',length(cases)),list(family='density',x=x,weight=seq_along(x),bounds=bounds,bw=.45,n=32,from=0,to=3),function()ggplot2:::compute_density(x,seq_along(x),0,3,bw=.45,n=32,bounds=bounds))
}
for(n in c(1,2,33,513))capture(paste0('density-grid-',n),list(family='density',x=x,bw=.45,n=n,from=-2,to=5),function()ggplot2:::compute_density(x,NULL,-2,5,bw=.45,n=n))
for(name in c('constant','singleton','missing')) {z=sets[[name]];capture(paste0('density-',name),list(family='density',x=z,n=32,from=0,to=4),function()ggplot2:::compute_density(z,NULL,0,4,n=32))}
for(w in list(rep(0,length(x)),rep(-1,length(x)),replace(rep(1,length(x)),1,NA)))capture(paste0('density-invalid-weight-',length(cases)),list(family='density',x=x,weight=w,bw=.45,n=32,from=-2,to=5),function()ggplot2:::compute_density(x,w,-2,5,bw=.45,n=32))
d=data.frame(g=rep(c('A','B'),c(4,7)),y=c(0,1,1,2,0,0,.5,1,1.5,2,4))
for(scale in c('area','count','width'))for(trim in c(FALSE,TRUE))capture(paste('violin',scale,trim,sep='-'),list(family='violin',data=d,scale=scale,trim=trim,bw=.5),function()built(ggplot(d,aes(g,y))+geom_violin(scale=scale,trim=trim,bw=.5)))
for(drop in c(FALSE,TRUE)){s=data.frame(g=c('A','B','B'),y=c(1,0,2));capture(paste0('violin-singleton-',drop),list(family='violin',data=s,drop=drop),function()built(ggplot(s,aes(g,y))+geom_violin(drop=drop,bw=.5)))}
dots=data.frame(x=c(0,.2,.9,1,1.1,2,2.2,3),y=c(0,.2,.9,1,1.1,2,2.2,3),g=rep(c('A','B'),4),w=c(1,2,0,1,2,1,1,1))
for(method in c('dotdensity','histodot'))for(binaxis in c('x','y'))for(binpositions in c('bygroup','all'))for(right in c(FALSE,TRUE))capture(paste('dot',method,binaxis,binpositions,right,sep='-'),list(family='dotplot',data=dots,method=method,binaxis=binaxis,binpositions=binpositions,right=right,binwidth=1),function()built(ggplot(dots,if(binaxis=="x")aes(x,group=g,weight=w)else aes(x=g,y=y,group=g,weight=w))+geom_dotplot(method=method,binaxis=binaxis,binpositions=binpositions,right=right,binwidth=1)))
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',dependencies=list(quantreg_available=requireNamespace('quantreg',quietly=TRUE),versions=setNames(lapply(c('quantreg','SparseM','MatrixModels','Matrix'),function(p)if(requireNamespace(p,quietly=TRUE))as.character(packageVersion(p))else NULL),c('quantreg','SparseM','MatrixModels','Matrix'))),cases=cases),'fixtures/parity/ggplot2/distribution-controls.json',auto_unbox=TRUE,digits=17,na='null',null='null')
cat('captured',length(cases),'cases\n')
# Independently show why a direct Gaussian sum is not the pinned FFT estimator.
z=cases[[which(vapply(cases,function(c)c$name=='density-gaussian-FALSE',logical(1)))]]
x=z$controls$x;d=z$result$value;direct=vapply(d$x,function(q)mean(dnorm(q-x,sd=.45)),numeric(1))
jsonlite::write_json(list(reference='R 4.6.1',x=d$x,fft_density=d$density,direct_density=direct,max_difference=max(abs(d$density-direct)),trapezoid_mass=sum(diff(d$x)*(head(d$density,-1)+tail(d$density,-1))/2)),'fixtures/parity/ggplot2/distribution-kernel-anchors.json',auto_unbox=TRUE,digits=17,pretty=TRUE)
