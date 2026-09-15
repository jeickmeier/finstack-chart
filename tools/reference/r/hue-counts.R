# FIX-GG04: independently sampled hue counts and nondefault polar-Luv parameters.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('scales'))=='1.4.0',as.character(packageVersion('farver'))=='2.1.2')
cases<-list()
for(n in c(1:32,64,128,255))for(mode in c('default','shifted','dark','reverse')){
 args<-switch(mode,default=list(),shifted=list(h=c(-100,720),c=65,l=40,h.start=33),dark=list(c=15,l=7.99),reverse=list(direction=-1))
 cases[[length(cases)+1L]]<-list(n=n,mode=mode,colors=as.list(do.call(scales::pal_hue,args)(n)))
}
jsonlite::write_json(list(reference='R 4.6.1 / scales 1.4.0 / farver 2.1.2',white_reference=as.list(farver:::as_white_ref('D65')),cases=cases),'fixtures/parity/ggplot2/hue-counts.json',auto_unbox=TRUE,pretty=TRUE,digits=17)
cat('PASS',length(cases),'hue count palettes\n')
