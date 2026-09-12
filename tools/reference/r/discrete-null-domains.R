# FIX-GG04: authored/factor missing levels retain identity and order independently of translation.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
populations<-list(mixed=c('a','b',NA,'NA'),finite=c('a','b'),missing=NA_character_,empty=character())
limits_list<-list(auto=NULL,first=c(NA,'b','a'),middle=c('b',NA,'a'),last=c('b','a',NA),only=NA_character_,empty=character(),finite=c('b','a'),unused=c('c',NA,'a'))
query<-c('a','b','c',NA,'NA','z');cases<-list()
for(kind in c('hue','identity'))for(population in names(populations))for(limit_name in names(limits_list))for(factor in c(FALSE,TRUE))for(drop in c(FALSE,TRUE))for(translate in c(FALSE,TRUE)){
 values<-populations[[population]];levels<-if(factor)c('b',NA,'a','NA','c')else NULL
 if(factor)values<-base::factor(values,levels=levels,exclude=NULL)
 s<-if(kind=='hue')scale_colour_hue(limits=limits_list[[limit_name]],drop=drop,na.translate=translate)else scale_discrete_identity(aesthetics='colour',guide='legend',limits=limits_list[[limit_name]],drop=drop,na.translate=translate)
 result<-tryCatch(suppressWarnings({s$train(values);list(breaks=as.list(unname(s$get_breaks())),labels=as.list(unname(s$get_labels())),guide_paints=as.list(unname(s$map(s$get_breaks()))),mapped=as.list(unname(s$map(query))),training_paints=as.list(unname(s$map(values))))}),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]<-list(kind=kind,population=population,inputs=as.list(as.character(values)),limits_name=limit_name,limits=if(is.null(limits_list[[limit_name]]))NULL else as.list(limits_list[[limit_name]]),levels=if(is.null(levels))NULL else as.list(levels),drop=drop,na_translate=translate,query=as.list(query),result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/discrete-null-domains.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'discrete nullable domain records\n')
