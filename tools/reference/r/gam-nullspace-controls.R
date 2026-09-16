# Isolate numerical nullspace orientation in the pinned cs shrinkage contract.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('mgcv'))=='1.9.4')
library(mgcv);base=seq(-2,2,length.out=61);y=sin(2*base)+.1*cos(9*base);cases=list()
for(shift in c(0,1e-8,1e-6,1e-4,1,10)){
 x=base+shift;k=10;cr=smoothCon(s(x,bs='cr',k=k),data=data.frame(x=x),absorb.cons=FALSE)[[1]];cs=smoothCon(s(x,bs='cs',k=k),data=data.frame(x=x),absorb.cons=FALSE)[[1]];raw=mgcv:::smooth.construct.cr.smooth.spec(s(x,bs="cr",k=k),data.frame(x=x),NULL)$S[[1]];shrunk=cs$S[[1]]*cs$S.scale;linear=cs$xp-mean(cs$xp);linear=linear/sqrt(sum(linear^2));constant=rep(1/sqrt(k),k);m=gam(y~s(x,bs='cs',k=k),method='REML');cases[[length(cases)+1]]=list(shift=shift,knots=cs$xp,raw=raw,shrunk=shrunk,eigenvalues=eigen(raw,symmetric=TRUE)$values,eigenvectors=eigen(raw,symmetric=TRUE)$vectors,linear_penalty=as.numeric(t(linear)%*%shrunk%*%linear),constant_penalty=as.numeric(t(constant)%*%shrunk%*%constant),prediction=as.numeric(predict(m,data.frame(x=seq(-3,3,length.out=11)+shift))))
}
jsonlite::write_json(list(reference='R 4.6.1 / mgcv 1.9.4',lapack=La_library(),lapack_version=La_version(),session=capture.output(sessionInfo()),cases=cases),'fixtures/parity/ggplot2/gam-nullspace-controls.json',auto_unbox=TRUE,digits=17)
