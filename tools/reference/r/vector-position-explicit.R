library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
encode <- function(x) lapply(unname(x),function(v) if(is.nan(v))list(number='NaN')else if(is.na(v))list(number='NA')else if(is.infinite(v))list(number=if(v>0)'Infinity'else'-Infinity')else v)
cases <- list();configs <- list()
for(family in c('cardinality','center'))for(composed in c(FALSE,TRUE)) {
 configs[[length(configs)+1L]] <- list(family=family,composed=composed)
 forward <- function(x)if(family=='cardinality')x+length(x)else x-mean(x,na.rm=TRUE)
 inverse <- function(x)if(family=='cardinality')x-length(x)else x
 tr <- scales::new_transform(family,forward,inverse)
 if(composed)tr <- tryCatch(scales::transform_compose(tr,scales::transform_reverse()),error=function(e)list(error=conditionMessage(e)))
 for(population in c('ordinary','nullable','empty'))for(layers in c('one','two'))for(mode in c('pair','empty','mixed'))for(label_mode in c('default','explicit')) {
  x <- switch(population,ordinary=c(1,2,4),nullable=c(1,NA_real_,4),empty=numeric());y <- if(population=='empty')numeric()else c(2,10)
  breaks <- switch(mode,pair=c(2,5),empty=numeric(),mixed=c(-3,-2,0,0,NA,Inf,-Inf))
  labels <- if(label_mode=='explicit')paste0('key-',seq_along(breaks))else waiver()
  if(label_mode=='explicit' && length(breaks)==0)labels <- character()
  result <- if(!is.null(tr$error))tr else tryCatch(suppressWarnings({
   p <- ggplot(data.frame(x=x),aes(x,1))+geom_point()+scale_x_continuous(transform=tr,breaks=breaks,labels=labels)
   if(layers=='two')p <- p+geom_point(data=data.frame(x=y))
   b <- ggplot_build(p);panel <- b$layout$panel_params[[1]]$x
   result <- list(mapped=encode(b$data[[1]]$x),range=encode(panel$continuous_range),breaks=encode(panel$get_breaks()),minor=encode(panel$minor_breaks),positions=encode(panel$break_positions()),labels=as.list(panel$get_labels()))
   if(layers=='two')result$mapped_second <- encode(b$data[[2]]$x)
   result
  }),error=function(e)list(error=conditionMessage(e)))
  cases[[length(cases)+1L]] <- list(configuration=length(configs)-1L,route='position',population=population,layers=layers,limits='auto',count=NULL,explicit_mode=mode,label_mode=label_mode,explicit_labels=if(label_mode=='explicit')as.list(labels)else NULL,break_values=encode(breaks),inputs=encode(x),second=if(layers=='two')encode(y)else NULL,result=result)
 }
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',configurations=configs,cases=cases),'fixtures/parity/ggplot2/vector-position-explicit.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat(length(cases),'vector position explicit builds captured\n')
