pdf(file = tempfile(fileext = '.pdf'))
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
cases <- list()
for (angle in c(0,45,90)) for (just in c(0,0.5,1)) {
 d <- data.frame(x=c(1,1,2), y=c(1,1,2), label=c('alpha','éλ',''))
 p <- ggplot(d,aes(x,y,label=label))+geom_text(angle=angle,hjust=just,vjust=just,size=10,size.unit='pt',check_overlap=TRUE)
 b <- ggplot_build(p)
 cases[[length(cases)+1]] <- list(angle=angle,hjust=just,vjust=just,labels=unname(b$data[[1]]$label),size=unname(b$data[[1]]$size),rows=nrow(b$data[[1]]))
}
p <- ggplot(data.frame(x=c('A','A','B')),aes(x,label=after_stat(count)))+geom_text(stat='count')
b <- ggplot_build(p)
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1', cases=cases, count=list(labels=unname(b$data[[1]]$label),counts=unname(b$data[[1]]$count)), units=as.list(setNames(vapply(c("mm","pt","cm","in","pc"),ggplot2:::resolve_text_unit,numeric(1)),c("mm","pt","cm","in","pc")))), 'fixtures/parity/ggplot2/text-marks.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null')
