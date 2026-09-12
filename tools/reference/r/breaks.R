# FIX-GG04: independent default-break and binned-policy oracle.
stopifnot(as.character(packageVersion('labeling'))=='0.4.3',as.character(packageVersion('scales'))=='1.4.0')
cases<-list()
for(domain in list(c(0,1),c(1,7),c(-3,17),c(-100,-1),c(.13,.79),c(1.123,1.143),c(1e12,1e12+100),c(1e-20,2e-20),c(1e150,2e150),c(1,1),c(10,0))) for(n in c(2,3,4,5,7,10,17)) {
 cases[[length(cases)+1]]<-list(kind='extended',domain=domain,count=n,breaks=as.list(scales::breaks_extended(n)(domain)))
}
set.seed(1943)
for(i in 1:80){lo<-runif(1,-100,100);hi<-lo+10^runif(1,-3,3);n<-sample(2:12,1);cases[[length(cases)+1]]<-list(kind='extended',domain=c(lo,hi),count=n,breaks=as.list(scales::breaks_extended(n)(c(lo,hi))))}
for(base in c(2,3,10,2.5))for(domain in list(c(.1,100),c(1,7),c(3,4),c(.123,.987),c(1,1),c(1e-6,1e6))) for(n in c(2,3,5,10)) {
 cases[[length(cases)+1]]<-list(kind='log',domain=domain,count=n,base=base,breaks=as.list(scales::breaks_log(n,base)(domain)))
}
for(domain in list(c(0,1),c(1,1),c(1e-20,2e-20),c(1.123,1.143))) for(n in c(1.5,2.5,3.1,7.9)) cases[[length(cases)+1]]<-list(kind='extended',domain=domain,count=n,breaks=as.list(scales::breaks_extended(n)(domain)))
for(domain in list(c(.1,100),c(3,4),c(1,1)))for(n in c(.5,1,1.5,2.5,7.9))cases[[length(cases)+1]]<-list(kind='log',domain=domain,count=n,base=10,breaks=as.list(scales::breaks_log(n,10)(domain)))
jsonlite::write_json(list(reference='scales 1.4.0 / labeling 0.4.3 / R 4.6.1 r90187',cases=cases),'fixtures/parity/ggplot2/breaks.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'break candidates\n')
