# FIX-GG04: alpha stays raw through scale mapping and is lowered by the paint owner.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
column<-function(x)lapply(seq_along(x),function(i){v<-x[[i]];if(is.na(v))return(list(number=if(is.nan(v))'NaN'else'NA'));if(is.numeric(v)&&!is.finite(v))return(list(number=if(v>0)'Infinity'else'-Infinity'));unname(v)})
alpha<-c(-Inf,-2,-.5,0,.1,.5,1,1.5,2,Inf,NA,NaN)
cases<-list()
for(after in c(FALSE,TRUE)) {
 p<-ggplot(data.frame(x=seq_along(alpha),a=alpha),aes(x,1,alpha=a))+geom_point(colour='#12345678')+scale_alpha_identity()
 if(after)p<-ggplot(data.frame(x=seq_along(alpha),a=alpha),aes(x,1,alpha=stage(a,after_scale=alpha*2)))+geom_point(colour='#12345678')+scale_alpha_identity()
 b<-ggplot_build(p);g<-ggplot_gtable(b);panel<-g$grobs[[which(g$layout$name=='panel')]];points<-panel$children[[which(vapply(panel$children,inherits,logical(1),'points'))]]
 cases[[length(cases)+1]]<-list(after_scale_double=after,input=column(alpha),mapped=column(b$data[[1]]$alpha),rgba=unname(split(t(grDevices::col2rgb(points$gp$col,alpha=TRUE)),row(t(grDevices::col2rgb(points$gp$col,alpha=TRUE))))))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/identity-alpha.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'alpha grobs /',length(alpha)*length(cases),'paint cases\n')
