# FIX-GG04: identity scales preserve raw aesthetic values; guide training is separate.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
column<-function(x)lapply(seq_along(x),function(i){v<-x[[i]];if(is.na(v))return(list(number=if(is.nan(v))'NaN'else'NA'));if(is.numeric(v)&&!is.finite(v))return(list(number=if(v>0)'Infinity'else'-Infinity'));unname(v)})
cases<-list()
for (kind in c('colour','fill','size','alpha','linewidth','shape','linetype')) for(guide in c('none','legend')) {
 categorical<-kind%in%c('colour','fill','linetype')
 train<-switch(kind,colour=c('red4','gray','green',NA_character_),fill=c('red4','gray','green',NA_character_),linetype=c('solid','dashed','dotted',NA_character_),c(.5,1,3,NA_real_))
 input<-switch(kind,colour=c('red4','blue','transparent','#ABC0',NA_character_),fill=c('red4','blue','transparent','#ABC0',NA_character_),linetype=c('solid','dotted','dotdash',NA_character_),c(-1,0,.5,1,3,10,Inf,NA_real_,NaN))
 scale<-get(paste0('scale_',kind,'_identity'),asNamespace('ggplot2'))(guide=guide)
 result<-tryCatch({scale$train(scale$transform(train));list(domain=column(scale$get_limits()),mapped=column(scale$map(scale$transform(input))),breaks=column(scale$get_breaks()))},error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,guide=guide,train=column(train),input=column(input),result=result)
}
for (kind in c('size','alpha','linewidth')) for(transform in c('identity','sqrt','log10','reverse')) {
 scale<-get(paste0('scale_',kind,'_identity'),asNamespace('ggplot2'))(limits=c(1,3),transform=transform,guide='none')
 input<-c(.1,1,2,4,10)
 scale$train(scale$transform(input))
 cases[[length(cases)+1]]<-list(kind=kind,guide='none',transform=transform,limits=as.list(c(1,3)),train=column(input),input=column(input),result=list(domain=column(scale$get_limits()),mapped=column(scale$map(scale$transform(input)))))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/identity-scales.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'identity scale records\n')
