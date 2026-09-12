# FIX-GG04: discrete minors use numeric category coordinates and censor to the expanded panel range.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.nan(x))'NaN'else if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else unname(x))
cases<-list()
for(n in c(0,1,3))for(expand_name in c('default','none','wide'))for(minor_name in c('auto','hidden','empty','explicit')){
 values<-head(letters,n)
 expansion_value<-switch(expand_name,default=waiver(),none=expansion(0),wide=expansion(mult=c(.2,.5),add=c(1,2)))
 minor_value<-switch(minor_name,auto=waiver(),hidden=NULL,empty=numeric(),explicit=c(-Inf,-1,0,.5,1,1.5,2.5,3.5,4,Inf,NA))
 result<-tryCatch(suppressWarnings({
  b<-ggplot_build(ggplot(data.frame(x=factor(values,levels=values),y=seq_len(n)),aes(x,y))+geom_point()+scale_x_discrete(expand=expansion_value,minor_breaks=minor_value))
  p<-b$layout$panel_params[[1]]$x;position<-p$rescale(p$minor_breaks);keep<-is.finite(p$minor_breaks)&is.finite(position)
  list(values=encode(p$minor_breaks[keep]),positions=encode(position[keep]),range=encode(p$continuous_range))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(count=n,expansion=expand_name,minor=minor_name,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/minor-discrete.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete minor panels\n')
