# GG09 development oracle for the intercept-only weighted boxplot estimator.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('quantreg'))=='6.1')
library(quantreg)
taus=c(0,.25,.5,.75,1)
cases=list()
add=function(name,y,w){warnings=character();result=withCallingHandlers(tryCatch({fit=rq(y~1,tau=taus,weights=w);q=as.numeric(coef(fit));list(ok=TRUE,quantiles=q,objective=vapply(seq_along(taus),function(i){r=y-q[i];sum(w*r*(taus[i]-(r<0)))},numeric(1)))},error=function(e)list(ok=FALSE,error=conditionMessage(e))),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')});keep=w>0;sy=y[keep];sw=w[keep];o=order(sy);sy=sy[o];sw=sw[o];proposal=if(length(sw)&&all(w>=0))vapply(taus,function(t){sy[which(cumsum(sw)>=t*sum(sw))[1]]},numeric(1))else NULL;cases[[length(cases)+1]]<<-list(name=name,y=y,weight=w,tau=taus,result=result,lower_weighted_inverse=proposal,warnings=warnings)}
sets=list(even=list(y=c(0,1,4,10),w=rep(1,4)),odd=list(y=c(-2,0,1,4,10),w=rep(1,5)),ties=list(y=c(0,0,1,1,3,3),w=c(1,2,1,2,1,1)),zero_ends=list(y=c(-20,0,1,4,10,30),w=c(0,1,1,1,1,0)),zero_inside=list(y=c(0,1,4,10),w=c(1,0,2,1)),all_zero=list(y=c(0,1,4,10),w=rep(0,4)),fractional=list(y=c(0,1,4,10),w=c(.1,.2,.3,.4)),negative=list(y=c(0,1,4,10),w=c(1,-1,2,1)),singleton=list(y=4,w=1),constant=list(y=rep(2,4),w=c(1,0,2,1)))
for(name in names(sets)){z=sets[[name]];n=length(z$y);orders=if(n==1)list(1)else list(seq_len(n),rev(seq_len(n)),c(seq.int(2,n,by=2),seq.int(1,n,by=2)));for(i in seq_along(orders)){o=orders[[i]];add(paste0(name,'-permutation-',i),z$y[o],z$w[o])}}
# Enumerate all 3^3 small integer weight vectors, including zero mass and exact ties.
for(i in 0:26){w=c(i%%3,(i%/%3)%%3,(i%/%9)%%3);for(o in list(1:3,3:1))add(paste0('enumerated-',i,'-',o[1]),c(-1,0,3)[o],w[o])}
jsonlite::write_json(list(reference=list(R=as.character(getRversion()),quantreg=as.character(packageVersion('quantreg'))),cases=cases),'fixtures/parity/ggplot2/weighted-intercept-quantiles.json',auto_unbox=TRUE,digits=17,na='null',null='null',pretty=TRUE)
cat('captured',length(cases),'weighted intercept cases\n')
