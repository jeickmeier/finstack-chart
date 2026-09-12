# FIX-GG04: transformed minor candidates and final drawable panel positions.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(v,function(x)if(is.nan(x))'NaN'else if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else unname(x))
major_sets<-list(auto=waiver(),regular=c(1,2,4,10),descending=c(10,4,2,1),outside=c(-5,0,2,20),one=4,empty=numeric(),none=NULL)
minor_sets<-list(auto=waiver(),explicit=c(-Inf,-2,0,.5,1,3,6,10,12,Inf,NA),empty=numeric(),none=NULL)
cases<-list()
for(trans in c('identity','sqrt','log10','reverse'))for(population in c('finite','constant','negative','zero_wide'))for(major in names(major_sets))for(minor in names(minor_sets)){
 input<-switch(population,finite=c(1,10),constant=c(4,4),negative=c(-4,10),zero_wide=c(0,10))
 result<-tryCatch(suppressWarnings({
  p<-ggplot(data.frame(x=input,y=c(1,2)),aes(x,y))+geom_point()+scale_x_continuous(transform=trans,breaks=major_sets[[major]],minor_breaks=minor_sets[[minor]],expand=if(population=='zero_wide')expansion(mult=1)else waiver())
  b<-ggplot_build(p);panel<-b$layout$panel_params[[1]]$x;t<-panel$scale$get_transformation()
  minor_positions<-panel$rescale(panel$minor_breaks)
  keep<-is.finite(panel$minor_breaks)&is.finite(minor_positions)
  majors<-panel$breaks[is.finite(panel$breaks)]
  list(range=encode(panel$continuous_range),major=encode(majors),minor=encode(panel$minor_breaks),minor_values=encode(t$inverse(panel$minor_breaks[keep])),minor_positions=encode(minor_positions[keep]))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(transform=trans,population=population,major=major,minor=minor,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/minor-breaks.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'numeric minor-break panels\n')
