# FIX-GG04 / GG2-01: reference paint-stage arithmetic retains infinities.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
column<-function(x)lapply(seq_along(x),function(i){v<-x[[i]];if(is.na(v))return(list(number=if(is.nan(v))'NaN'else'NA'));if(is.numeric(v)&&!is.finite(v))return(list(number=if(v>0)'Infinity'else'-Infinity'));unname(v)})
alpha<-c(-Inf,-2,-.5,0,.1,.5,1,1.5,2,Inf,NA,NaN)
expressions<-c(negate='-alpha',abs='abs(alpha)',sqrt='sqrt(alpha)',log='log(alpha)',exp='exp(alpha)',reciprocal='1/alpha',subtract='alpha-alpha',power='alpha^0',sum='sum(alpha,na.rm=TRUE)',mean='mean(alpha,na.rm=TRUE)',min='min(alpha,na.rm=TRUE)',max='max(alpha,na.rm=TRUE)',abs_sum='sum(abs(alpha),na.rm=TRUE)',missing_sum='sum(alpha,na.rm=FALSE)')
cases<-list()
for(name in names(expressions)) {
 expr<-rlang::parse_expr(expressions[[name]])
 p<-ggplot(data.frame(x=seq_along(alpha),a=alpha),aes(x,1,alpha=stage(a,after_scale=!!expr)))+geom_point(colour='#12345678')+scale_alpha_identity()
 b<-suppressWarnings(ggplot_build(p));g<-ggplot_gtable(b);panel<-g$grobs[[which(g$layout$name=='panel')]];points<-panel$children[[which(vapply(panel$children,inherits,logical(1),'points'))]]
 rgba<-t(grDevices::col2rgb(rep_len(points$gp$col,length(alpha)),alpha=TRUE))
 cases[[length(cases)+1]]<-list(name=name,expression=expressions[[name]],input=column(alpha),mapped=column(rep_len(b$data[[1]]$alpha,length(alpha))),rgba=unname(split(rgba,row(rgba))))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/identity-alpha-expressions.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'alpha expression grobs /',length(cases)*length(alpha),'paints\n')
