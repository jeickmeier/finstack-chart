stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases=list()
for(width in c(-1,0,1))for(constant in c(FALSE,TRUE)){
 x=if(constant)c(2,2,2)else c(1,2,3);b=ggplot_build(ggplot(data.frame(x=x),aes(x))+geom_dotplot(binwidth=width))$data[[1]];cases[[length(cases)+1]]=list(width=width,x=x,values=b[,c('x','count','binwidth')])
}
jsonlite::write_json(list(reference='R4.6.1 / ggplot2 4.0.3',cases=cases),'fixtures/parity/ggplot2/dot-width-controls.json',auto_unbox=TRUE,digits=17,pretty=TRUE)
