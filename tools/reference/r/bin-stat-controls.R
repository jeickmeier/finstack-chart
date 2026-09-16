# GG-06 pinned bin closure/alignment/weight/normalization contract.
library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
cases <- list()
for (closed in c('right','left')) for (pad in c(FALSE,TRUE))
 for (population in c('ordinary','boundary','constant','signed','zero'))
  for (mode in c('explicit','bins','width','center','boundary')) {
  x <- switch(population, ordinary=c(0,0.5,1,2,3),boundary=c(0,1-1e-9,1,1+1e-9,2),constant=rep(1,5),signed=c(0,.5,1,2,3),zero=c(0,.5,1,2,3))
  weight <- switch(population,signed=c(1,-2,3,-4,5),zero=rep(0,5),c(1,2,0,4,1))
  limits <- range(x)
  b <- switch(mode,explicit=ggplot2:::bin_breaks(c(0,1,2,4),closed),bins=ggplot2:::bin_breaks_bins(limits,3,closed=closed),width=ggplot2:::bin_breaks_width(limits,.75,closed=closed),center=ggplot2:::bin_breaks_width(limits,.75,center=.25,closed=closed),boundary=ggplot2:::bin_breaks_width(limits,.75,boundary=.25,closed=closed))
  out <- ggplot2:::bin_vector(x,b,weight,pad)
  cases[[length(cases)+1]] <- list(closed=closed,pad=pad,population=population,mode=mode,input=as.list(x),weight=as.list(weight),breaks=as.list(b$breaks),fuzzy=as.list(b$fuzzy),result=out)
 }
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/bin-stat-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null',na='null')
cat('PASS',length(cases),'bin-stat control cases\n')
