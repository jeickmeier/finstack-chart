# FIX-GG04: identity guide controls never alter the raw mapping.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
column<-function(x)lapply(seq_along(x),function(i){v<-x[[i]];if(is.na(v))return(list(number=if(is.nan(v))'NaN'else'NA'));if(is.numeric(v)&&!is.finite(v))return(list(number=if(v>0)'Infinity'else'-Infinity'));unname(v)})
library(ggplot2)
numeric<-list();discrete<-list()
for(transform in c('identity','sqrt','log10','reverse')) for(guide in c('none','legend')) for(limits in list(NULL,c(NA,3),c(1,NA),c(3,1),c(-2,3),c(0,3))) {
 train<-c(-2,0,.5,1,3,10,Inf,NA,NaN)
 s<-suppressWarnings(scale_size_identity(transform=transform,limits=limits,guide=guide))
 suppressWarnings(s$train(s$transform(train)))
 numeric[[length(numeric)+1]]<-list(transform=transform,guide=guide,limits=if(is.null(limits))NULL else column(limits),train=column(train),domain=column(s$get_limits()),mapped=column(suppressWarnings(s$map(s$transform(train)))))
}
for(kind in c('colour','fill','linetype')) for(guide in c('none','legend')) for(drop in c(TRUE,FALSE)) for(na_translate in c(TRUE,FALSE)) for(limits in list(NULL,c('blue','red4'))) {
 train<-factor(c('red4','gray',NA),levels=c('red4','blue','gray','green'))
 s<-get(paste0('scale_',kind,'_identity'),asNamespace('ggplot2'))(guide=guide,drop=drop,na.translate=na_translate,limits=limits)
 s$train(train)
 input<-c('blue','unused',NA)
 discrete[[length(discrete)+1]]<-list(kind=kind,guide=guide,drop=drop,na_translate=na_translate,limits=limits,levels=as.list(levels(train)),train=column(as.character(train)),domain=column(s$get_limits()),input=column(input),mapped=column(s$map(input)))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',numeric=numeric,discrete=discrete),'fixtures/parity/ggplot2/identity-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(numeric),'numeric and',length(discrete),'discrete identity control cases\n')
