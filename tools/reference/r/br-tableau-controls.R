# Original shared Rust BR solver qualification against an external pinned oracle.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('quantreg'))=='6.1')
library(quantreg)
cases=list()
x=seq(-3,3,length.out=17);y=1+2*x-.4*x*x+sin(3*x);w=1+(seq_along(x)%%3)
for(kind in c('linear','quadratic','duplicate','near_collinear'))for(tau in c(.1,.5,.9)){
 X=switch(kind,linear=cbind(1,x),quadratic=cbind(1,x,x*x),duplicate=cbind(1,x,2*x),near_collinear=cbind(1,x,x+1e-9*x*x));warnings=character();result=withCallingHandlers(tryCatch({fit=rq.wfit(X,y,tau=tau,weights=w,method='br');r=as.vector(y-X%*%fit$coefficients);list(ok=TRUE,coefficients=fit$coefficients,residuals=r,objective=sum(w*r*(tau-(r<0))))},error=function(e)list(ok=FALSE,error=conditionMessage(e))),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')});cases[[length(cases)+1]]=list(name=paste(kind,tau,sep='-'),design=unname(X),response=y,weight=w,tau=tau,result=result,warnings=warnings)
}
jsonlite::write_json(list(reference=list(R=as.character(getRversion()),quantreg=as.character(packageVersion('quantreg'))),cases=cases),'fixtures/parity/ggplot2/br-tableau-controls.json',auto_unbox=TRUE,digits=17,pretty=TRUE)
