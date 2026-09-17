# Independent GG11 statistic/topology contract, using pinned reference implementations.
library(ggplot2)
library(jsonlite)
stopifnot(as.character(packageVersion('ggplot2')) == '4.0.3', as.character(packageVersion('hexbin')) == '1.28.6', as.character(packageVersion('isoband')) == '0.3.0')
cases <- list()
capture <- function(name, data, layer, mapping=aes(x,y), controls=list()) {
  warnings <- character()
  result <- tryCatch(withCallingHandlers({
    p <- ggplot(data,mapping)+layer
    b <- ggplot_build(p)
    ggplotGrob(p) # Exercise draw/topology, not only numeric output.
    list(built=lapply(b$data[[1]],function(x) if(is.factor(x)) as.character(x) else unclass(x)))
  }, warning=function(w) {warnings <<- c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
  cases[[length(cases)+1]] <<- c(list(name=name,input=data,controls=controls,warnings=unique(warnings)),result)
}
d <- data.frame(x=c(0,.25,.5,1,1.5,2,2),y=c(0,.75,1,.5,1.5,2,0),z=c(1,2,4,8,16,32,64),w=c(1,2,0,3,1,2,1))
for(closed in c('left','right')) for(drop in c(TRUE,FALSE)) capture(paste('bin2d',closed,drop,sep='-'),d,geom_bin_2d(binwidth=c(1,1),boundary=c(0,0),closed=closed,drop=drop),aes(x,y,weight=w),list(binwidth=c(1,1),boundary=c(0,0),closed=closed,drop=drop))
capture('bin2d-breaks',d,geom_bin_2d(breaks=list(x=c(-1,.5,3),y=c(-1,1,3))),aes(x,y,weight=w))
for(fun in c('mean','sum','median')) capture(paste0('summary2d-',fun),d,stat_summary_2d(binwidth=c(1,1),fun=fun,drop=FALSE),aes(x,y,z=z),list(fun=fun))
capture('hex-count',d,geom_hex(binwidth=c(1,1)),aes(x,y,weight=w),list(binwidth=c(1,1)))
for(fun in c('mean','sum','median')) capture(paste0('summaryhex-',fun),d,stat_summary_hex(binwidth=c(1,1),fun=fun),aes(x,y,z=z),list(fun=fun))
for(weighted in c(FALSE,TRUE)) capture(paste0('kde2d-weight-',weighted),d,stat_density_2d(h=c(1,2),n=5,contour=FALSE),if(weighted) aes(x,y,weight=w) else aes(x,y),list(h=c(1,2),n=5))
capture('kde2d-default-bandwidth',d,stat_density_2d(n=5,contour=FALSE))
capture('kde2d-adjust',d,stat_density_2d(n=5,adjust=c(.5,2),contour=FALSE))
g <- expand.grid(x=-2:2,y=-2:2);g$z <- g$x^2+g$y^2
capture('contour-rings',g,geom_contour(breaks=c(1,2,4)),aes(x,y,z=z))
capture('isoband-hole',g,geom_contour_filled(breaks=c(.5,2,4)),aes(x,y,z=z))
s <- expand.grid(x=0:1,y=0:1);s$z <- c(0,2,2,0)
capture('contour-saddle',s,geom_contour(breaks=c(.5,1,1.5)),aes(x,y,z=z))
capture('isoband-saddle',s,geom_contour_filled(breaks=c(.5,1,1.5)),aes(x,y,z=z))
g$z[13] <- NA_real_
capture('contour-missing-center',g,geom_contour(breaks=c(1,2,4)),aes(x,y,z=z))
capture('isoband-missing-center',g,geom_contour_filled(breaks=c(.5,2,4)),aes(x,y,z=z))
for(type in c('norm','t','euclid')) capture(paste0('ellipse-',type),d,stat_ellipse(type=type,level=.8,segments=8),aes(x,y,weight=w),list(type=type,level=.8,segments=8))
capture('ellipse-too-few',d[1:3,],stat_ellipse(type='norm'))
capture('ellipse-degenerate',data.frame(x=1:5,y=1:5),stat_ellipse(type='norm'))
missing <- d;missing$z[2] <- NA_real_;missing$w[2] <- NA_real_
capture('summary2d-missing',missing,stat_summary_2d(binwidth=c(1,1),fun='mean',drop=FALSE),aes(x,y,z=z))
capture('summaryhex-missing',missing,stat_summary_hex(binwidth=c(1,1),fun='mean'),aes(x,y,z=z))
capture('bin2d-missing-weight',missing,geom_bin_2d(binwidth=c(1,1)),aes(x,y,weight=w))
capture('hex-missing-weight',missing,geom_hex(binwidth=c(1,1)),aes(x,y,weight=w))
rot <- expand.grid(x=-2:2,y=-2:2);rot$z<-rot$x^2+rot$y^2
q<-ggplot2:::rotate_xy(rot$x,rot$y,.37);rot$x<-q$x;rot$y<-q$y
capture('contour-rotated',rot,geom_contour(breaks=c(1,2,4)),aes(x,y,z=z))
capture('isoband-rotated',rot,geom_contour_filled(breaks=c(.5,2,4)),aes(x,y,z=z))
writeLines(toJSON(list(reference=paste('ggplot2',packageVersion('ggplot2'),'R',getRversion()),cases=cases),auto_unbox=TRUE,digits=17,na='null'),'fixtures/parity/ggplot2/spatial-statistics-controls.json')
