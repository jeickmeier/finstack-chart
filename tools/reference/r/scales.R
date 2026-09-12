# FIX-GG04: direct scale training/mapping plus plot defaults, under the pinned R runtime.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
column <- function(x) lapply(seq_along(x),function(i){v<-x[[i]];if(is.na(v))return(list(number=if(is.nan(v))'NaN' else 'NA'));if(is.numeric(v)&&!is.finite(v))return(list(number=if(v>0)'Infinity' else '-Infinity'));unname(v)})
cases<-list()
record<-function(id,kind,args,scale,train,input){
 warnings<-character()
 result<-tryCatch(withCallingHandlers({scale$train(scale$transform(train));if(kind=='binned')scale$get_breaks();limits<-scale$get_limits();list(domain=column(limits),mapped=column(scale$map(scale$transform(input))),breaks=column(scale$get_breaks()))},warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<<-list(id=id,kind=kind,args=args,train=column(train),input=column(input),result=result,warnings=as.list(warnings))
}
input<-c(-Inf,-10,-1,0,.1,1,3,7,10,20,100,Inf,NA_real_)
for(limits in list(NULL,c(0,10),c(NA,10),c(0,NA))) for(oob in c('censor','censor_any','squish','squish_any','keep','squish_infinite')) {
 f<-get(paste0('oob_',oob),asNamespace('scales'))
 args<-list(limits=limits,oob=oob)
 record(paste('continuous',paste(limits,collapse=':'),oob),'continuous',args,scale_colour_gradient(limits=limits,oob=f),c(1,3,7,NA,Inf),input)
}
for(transform in c('log10','sqrt','reverse')) record(paste('transform',transform),'transformed',list(transform=transform),scale_colour_gradient(transform=transform),c(.1,1,10,100),input)
for(drop in c(TRUE,FALSE)) for(na_translate in c(TRUE,FALSE)) for(factor in c(TRUE,FALSE)) {
 train<-c('b','a','b',NA_character_)
 if(factor) train<-base::factor(train,levels=c('c','b','a'))
 args<-list(drop=drop,na_translate=na_translate,levels=if(factor)c('c','b','a') else NULL)
 record(paste('discrete',drop,na_translate,factor),'discrete',args,scale_colour_hue(drop=drop,na.translate=na_translate),train,c('a','b','c','unknown',NA_character_))
}
for(named in c(TRUE,FALSE)) for(limits in list(NULL,c('b','a'))) {
 values<-if(named)c(b='red',a='blue') else c('red','blue')
 record(paste('manual',named,paste(limits,collapse=':')),'manual',list(named=named,limits=limits,values=values),scale_colour_manual(values=values,limits=limits),c('a','c','b'),c('a','b','c',NA_character_))
}
for(kind in c('size','size_area','radius','alpha','linewidth')) for(limits in list(NULL,c(0,16))) {
 constructor<-get(paste0('scale_',kind),asNamespace('ggplot2'))
 s<-constructor(limits=limits)
 if(is.null(s$palette))s$palette<-s$fallback_palette
 record(paste('sizing',kind,paste(limits,collapse=':')),'sizing',list(kind=kind,limits=limits),s,c(1,4,9,16),c(0,1,4,9,16,20,NA_real_))
}
for(limits in list(NULL,c(0,8),c(NA,8))) for(right in c(TRUE,FALSE)) for(mode in c('nice','equal','fixed','outside','none')) {
 breaks<-switch(mode,nice=waiver(),equal=waiver(),fixed=c(2,4,6),outside=c(-1,2,4,9),none=NULL)
 args<-list(limits=limits,right=right,mode=mode,breaks=if(is_waiver(breaks))NULL else breaks,n=5)
 record(paste('binned',paste(limits,collapse=':'),right,mode),'binned',args,scale_colour_steps(limits=limits,right=right,breaks=breaks,nice.breaks=mode!='equal',n.breaks=5),c(1.2,3,6.7),c(-Inf,-1,0,1,1.2,2,3,4,6,6.7,7,8,9,Inf,NA_real_))
}
for(kind in c('size','alpha','linewidth')) for(n in c(0,1,3)) {
 s<-get(paste0('scale_',kind,'_ordinal'),asNamespace('ggplot2'))()
 if(is.null(s$palette))s$palette<-s$fallback_palette
 record(paste('ordinal',kind,n),'ordinal',list(kind=kind),s,letters[seq_len(n)],c('a','b','c',NA_character_))
}
for(oob in c('censor','squish','keep')) {
 f<-get(paste0('oob_',oob),asNamespace('scales'))
 record(paste('constant',oob),'continuous',list(limits=c(3,3),oob=oob),scale_colour_gradient(limits=c(3,3),oob=f),c(3,3),c(-Inf,0,3,7,Inf,NA_real_))
}
for(right in c(TRUE,FALSE)) record(paste('constant-bin',right),'binned',list(limits=c(3,3),right=right,mode='none',n=5),scale_colour_steps(limits=c(3,3),right=right,breaks=NULL),c(3,3),c(-Inf,0,3,7,Inf,NA_real_))
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1 r90187',cases=cases),'fixtures/parity/ggplot2/scales.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'scale policy records\n')
