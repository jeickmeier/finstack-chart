# FIX-GG04: untrained lookup defaults are separate from an empty trained palette.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases<-list()
for(palette in c('hue','manual_one','manual_two','named'))for(population in c('empty','missing'))for(translate in c(FALSE,TRUE))for(factor in c(FALSE,TRUE))for(explicit_breaks in c(FALSE,TRUE)){
 values<-if(population=='empty')character()else NA_character_;levels<-if(factor)c('0','1','2')else NULL
 if(factor)values<-base::factor(values,levels=levels)
 breaks<-if(explicit_breaks)c('1','0')else waiver()
 s<-if(palette=='hue')scale_colour_hue(na.translate=translate,breaks=breaks,drop=FALSE)else scale_colour_manual(values=switch(palette,manual_one='#112233',manual_two=c('#112233','#445566'),named=c('0'='#112233','1'='#445566')),na.translate=translate,breaks=breaks,drop=FALSE)
 s$train(values)
 result<-tryCatch(list(text=as.list(unname(s$map(c('0','1','2',NA)))),numeric=as.list(unname(s$map(c(0,1,2,NA)))),logical=as.list(unname(s$map(c(FALSE,TRUE,NA))))),error=function(e)list(error=conditionMessage(e)))
 if(population=='empty') result$empty_chart_ok<-tryCatch({suppressWarnings(ggplotGrob(ggplot(data.frame(x=numeric(),v=values),aes(x,1,colour=v))+geom_point()+s$clone()));TRUE},error=function(e)FALSE)
 cases[[length(cases)+1]]<-list(palette=palette,population=population,inputs=as.list(as.character(values)),levels=if(is.null(levels))NULL else as.list(levels),translate=translate,breaks=if(explicit_breaks)as.list(breaks)else NULL,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/untrained-discrete-lookups.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'untrained discrete lookup records\n')
