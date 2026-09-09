# FIX-GG02 independent stage/group/limit oracle; the locked GG-00 runner supplies R.
stopifnot(as.character(getRversion())=="4.6.1",as.character(packageVersion("ggplot2"))=="4.0.3")
suppressPackageStartupMessages(library(ggplot2))
RNGkind("Mersenne-Twister","Inversion","Rejection");set.seed(1729)
args<-commandArgs(trailingOnly=TRUE);out<-if(length(args)) args[[1]] else "fixtures/parity/ggplot2/stages.json"
mean_data<-data.frame(x=1,y=c(1,10,100))
hist_data<-data.frame(x=c(1,2,5,20,50,200,500))
group_data<-data.frame(x=factor(c("X","X","Y","Y")),y=1:4,colour=factor(c("A","B","A","B")))
missing_data<-data.frame(x=1:4,y=1:4,colour=factor(c("A",NA,"B",NA)))
mean_plot<-ggplot(mean_data,aes(x,y))+stat_summary(fun=mean,geom="point")
hist_plot<-ggplot(hist_data,aes(x))+geom_histogram(breaks=c(1,10,100,1000),closed="left")
plots<-list(
 log_histogram=hist_plot+scale_x_log10(),
 horizontal_log_histogram=ggplot(hist_data,aes(y=x))+geom_histogram(breaks=c(1,10,100,1000),closed="left",orientation="y")+scale_y_log10(),
 horizontal_columns=ggplot(data.frame(category=factor(c("A","B")),value=c(3,7)),aes(y=category,x=value))+geom_col(orientation="y"),
 horizontal_count=ggplot(data.frame(category=factor(c("A","A","B"))),aes(y=category))+geom_bar(orientation="y"),
 coordinate_log_histogram=hist_plot+coord_transform(x="log10"),
 log_mean=mean_plot+scale_y_log10(),
 log_after_stat_expression=ggplot(mean_data,aes(x,y))+stat_summary(aes(y=stage(start=y,after_stat=y*2)),fun=mean,geom="point")+scale_y_log10(),
 coordinate_log_mean=mean_plot+coord_transform(y="log10"),
 scale_limit_mean=mean_plot+scale_y_continuous(limits=c(1,10)),
 coordinate_zoom_mean=mean_plot+coord_cartesian(ylim=c(1,10)),
 squish_mean=mean_plot+scale_y_continuous(limits=c(1,10),oob=scales::squish),
 keep_mean=mean_plot+scale_y_continuous(limits=c(1,10),oob=scales::oob_keep),
 multiple_discrete=ggplot(group_data,aes(x,y,colour=colour))+geom_point(),
 missing_groups=ggplot(missing_data,aes(x,y,colour=colour))+geom_point(),
 inferred_lines=ggplot(data.frame(x=c(1,2,1,2),y=c(1,3,2,4),colour=c("A","A","B","B")),aes(x,y,colour=colour))+geom_line(),
 numeric_groups=ggplot(data.frame(x=1:5,y=1:5,g=c(-0,0,1.5,1.5,NA)),aes(x,y,group=g))+geom_point(),
 source_reduction=ggplot(mean_data,aes(x,y=y/sum(y)))+geom_point(),
 stat_group_override=ggplot(mean_data,aes(x,y,group=factor(y)))+stat_summary(aes(group=1),fun=mean,geom="point"),
 source_expression=ggplot(mean_data,aes(x=x+1,y=y*2))+geom_point(),
 after_stat_expression=ggplot(hist_data,aes(x,y=after_stat(count/sum(count))))+geom_histogram(breaks=c(1,10,100,1000),closed="left"),
 after_scale_expression=ggplot(mean_data,aes(x,y,size=after_scale(size*2)))+geom_point(),
 theme_expression=ggplot(mean_data,aes(x,y))+geom_point(aes(colour=from_theme(accent)))+theme(geom=element_geom(accent="#1256ab"))
)
column<-function(x) lapply(seq_along(x),function(i){v<-x[[i]];if(is.na(v)) return(list(number=if(is.nan(v))"NaN"else"NA"));if(is.numeric(v)&&!is.finite(v)) return(list(number=if(v>0)"Infinity"else"-Infinity"));unname(v)})
cases<-lapply(names(plots),function(id){warnings<-character();built<-withCallingHandlers(ggplot_build(plots[[id]]),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart("muffleWarning")});list(id=id,owner="GG-02",layers=lapply(built$data,function(d)list(row_count=nrow(d),columns=lapply(d,column))),warnings=as.list(warnings))})
jsonlite::write_json(list(schema_version=1,reference="ggplot2 4.0.3",r_version=as.character(getRversion()),seed=1729,cases=cases),out,auto_unbox=TRUE,pretty=TRUE,digits=NA,null="null")
cat("PASS",length(cases),"FIX-GG02 independent stage/group oracle cases\n")
