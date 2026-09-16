# GG10 canonical Gaussian cs REML oracle. No runtime model dependency.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('mgcv'))=='1.9.4')
library(mgcv);cases=list()
for(mode in c('default','weighted','small-basis','linear','constant','ties','insufficient')){
 x=seq(-2,2,length.out=61);y=sin(2*x)+.1*cos(9*x);w=rep(1,length(x));k=10
 if(mode=='weighted')w=1+(seq_along(x)%%3)
 if(mode=='small-basis')k=6
 if(mode=='linear')y=2+3*x
 if(mode=='constant')y[]=2
 if(mode=='ties')x=round(x,1)
 if(mode=='insufficient')x=rep(1:4,length.out=61)
 warnings=character();result=withCallingHandlers(tryCatch({m=gam(y~s(x,bs='cs',k=k),weights=w,method='REML');sm=smoothCon(s(x,bs='cs',k=k),data=data.frame(x=x),absorb.cons=FALSE)[[1]];raw_object=mgcv:::smooth.construct.cr.smooth.spec(s(x,bs="cr",k=k),data.frame(x=x),NULL);raw=raw_object$S[[1]];list(raw_F=raw_object$F,raw_penalty=raw,raw_eigenvalues=eigen(raw,symmetric=TRUE)$values,raw_eigenvectors=eigen(raw,symmetric=TRUE)$vectors,ok=TRUE,prediction=predict(m,data.frame(x=seq(-3,3,length.out=11)),se.fit=TRUE),sp=m$sp,scale=m$sig2,edf=sum(m$edf),reml=as.numeric(m$gcv.ubre),knots=sm$xp,penalty=sm$S[[1]],penalty_scale=sm$S.scale,constraint=sm$C,model_design_knots=predict(m,data.frame(x=sm$xp),type="lpmatrix"),model_penalty=m$smooth[[1]]$S[[1]],model_scale=m$smooth[[1]]$S.scale)},error=function(e)list(ok=FALSE,error=conditionMessage(e))),warning=function(e){warnings<<-c(warnings,conditionMessage(e));invokeRestart('muffleWarning')});cases[[length(cases)+1]]=list(name=mode,data=data.frame(x=x,y=y,w=w),k=k,result=result,warnings=warnings)
}
jsonlite::write_json(list(reference='R 4.6.1 / mgcv 1.9.4',grid=seq(-3,3,length.out=11),cases=cases),'fixtures/parity/ggplot2/gam-controls.json',auto_unbox=TRUE,digits=17,na='null',null='null')
