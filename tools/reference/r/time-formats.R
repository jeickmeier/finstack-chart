# FIX-GG04: R C-locale date_labels, including first-only fractional directives.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
Sys.setenv(TZ='UTC'); Sys.setlocale('LC_TIME','C'); options(digits.secs=0)
patterns<-c('%Y-%m-%d %H:%M:%S','%a|%A|%b|%B|%h|%p|%P','%c|%x|%X','%C|%F|%D|%r|%R|%T|%k|%l|%v|%+', '%j|%u|%w|%U|%W|%V|%g|%G|%y|%Y','%z|%Z','%OS','%OS0','%OS1','%OS3','%OS6','%OS9','%OS3|%OS6','%OS10|%OS12|%OS00','%%OS3','%%%OS3','%%%%OS3','a%%OS3b','%_d|%-d|%0d|%q|%Q|%f|%L','%Ea|%EC|%Ey|%EY|%Ez|%Oa|%Od|%Om|%OI|%OH|%OM|%Ou|%OU|%OV|%Ow|%OW|%Oy|%OY','%n%t%%|abc%|%E|%O','%Y\n%b %d')
# Integer source microseconds are converted to POSIXct as integral seconds + fraction.
us<-c(1704164645123456,1704164645100000,1704164645123000,-400,100000,0,1609459200000000,1451606400000000,1483228800000000,951827696000000)
cases<-list()
for(value in us)for(pattern in patterns){
 sec<-floor(value/1e6)+(value%%1e6)/1e6
 x<-as.POSIXct(sec,origin='1970-01-01',tz='UTC')
 cases[[length(cases)+1]]<-list(microseconds=sprintf("%.0f",value),pattern=pattern,label=format(x,pattern))
}
jsonlite::write_json(list(reference='R 4.6.1 / ggplot2 4.0.3; LC_TIME=C; digits.secs=0; TZ=UTC',cases=cases),'fixtures/parity/ggplot2/time-formats.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'R time formats\n')
