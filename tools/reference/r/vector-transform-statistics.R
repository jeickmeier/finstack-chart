# FIX-GG04: transform generated count vectors after statistics.
suppressPackageStartupMessages(library(ggplot2))
stopifnot(as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
encode <- function(x) lapply(x,function(v) if(is.na(v))list(number='NaN')else if(is.infinite(v))list(number=if(v>0)'Infinity'else '-Infinity')else v)
cases <- list()
with_style <- Sys.getenv("VECTOR_STATISTIC_STYLES")=="1"
for(family in c('cardinality','center')) for(composed in c(FALSE,TRUE)) for(faceted in c(FALSE,TRUE)) for(route in if(with_style)c("paint","size")else "position") {
  d <- data.frame(x=c('a','a','b','c','c','c'),panel=c('a','b','a','a','b','b'))
  result <- tryCatch(suppressWarnings({
    tr <- if(family=='cardinality')scales::new_transform(family,function(x)x+length(x),function(x)x-length(x))else scales::new_transform(family,function(x)x-mean(x,na.rm=TRUE),identity)
    if(composed) tr <- scales::transform_compose(tr,scales::transform_reverse())
    p <- ggplot(d,aes(x,after_stat(count)))+geom_point(stat='count')+scale_y_continuous(transform=tr)
    if(route=='paint')p<-ggplot(d,aes(x,after_stat(count),colour=after_stat(count)))+geom_point(stat='count')+scale_colour_continuous(transform=tr)
    if(route=='size')p<-ggplot(d,aes(x,after_stat(count),size=after_stat(count)))+geom_point(stat='count')+scale_size_continuous(transform=tr)
    if(faceted)p<-p+facet_wrap(~panel)
    b<-ggplot_build(p)
    list(panels=lapply(split(b$data[[1]],b$data[[1]]$PANEL),function(rows)list(x=encode(as.numeric(rows$x)),y=encode(rows$y),mapped=if(route=='paint')as.list(rows$colour)else if(route=='size')encode(rows$size)else encode(rows$y))))
  }),error=function(e)list(error=conditionMessage(e)))
  cases[[length(cases)+1L]]<-list(route=route,family=family,composed=composed,faceted=faceted,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=cases),if(with_style)'fixtures/parity/ggplot2/vector-transform-statistic-styles.json'else 'fixtures/parity/ggplot2/vector-transform-statistics.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null')
cat('Captured',length(cases),'generated count charts; reference only.\n')
