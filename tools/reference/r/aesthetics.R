# FIX-GG03: pinned ggplot2 builds plus numerical calls to R's own graphics engine.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
suppressPackageStartupMessages(library(ggplot2))
args <- commandArgs(trailingOnly=TRUE)
out <- if(length(args)) args[[1]] else 'fixtures/parity/ggplot2/aesthetics.json'
dir.create('target/ggplot-aesthetics-reference', showWarnings=FALSE, recursive=TRUE)
configurations <- expand.grid(pch=0:25,device_size=c(2,8,16,32))
symbols <- lapply(seq_len(nrow(configurations)),function(i) {
 pch <- configurations$pch[i]; size <- configurations$device_size[i]
 file <- sprintf('target/ggplot-aesthetics-reference/pch-%02d-size-%02d.pdf',pch,size)
 pdf(file,width=2,height=2,compress=FALSE)
 grid::grid.points(x=grid::unit(72,'bigpts'),y=grid::unit(72,'bigpts'),pch=pch,
                  gp=grid::gpar(fontsize=size,col='blue',fill='red',lwd=1))
 dev.off()
 list(pch=pch,device_size=size,pdf=file)
})
d <- data.frame(x=1:4,y=c(1,2,1,2),a=factor(c('A','B','A','B')),b=factor(c('B','A','A','B')),size=c(1,4,9,16),alpha=c(.2,.4,.6,.8),width=c(.5,1,1.5,2))
base <- ggplot(d,aes(x,y,fill=a,colour=b))
plots <- list(
 independent_paints=base+geom_point(shape=21)+scale_fill_manual(values=c(A='#ff0000',B='#0000ff'))+scale_colour_manual(values=c(A='#000000',B='#008000')),
 constant_fill=base+geom_point(shape=21,fill='#123456'),
 constant_stroke=base+geom_point(shape=21,colour='#123456'),
 mapped_alpha=base+geom_point(aes(alpha=alpha),shape=21)+scale_alpha_identity(),
 embedded_alpha=ggplot(d,aes(x,y,alpha=alpha))+geom_point(shape=21,fill='#ff000080',colour='#00000080')+scale_alpha_identity(),
 area_size=ggplot(d,aes(x,y,size=size))+geom_point()+scale_size_area(max_size=8),
 radius_size=ggplot(d,aes(x,y,size=size))+geom_point()+scale_radius(range=c(1,16),limits=c(1,16)),
 linewidth=ggplot(d,aes(x,y,linewidth=width,group=1))+geom_line()+scale_linewidth_identity(),
 constant_linewidth=ggplot(d,aes(x,y,linewidth=width,group=1))+geom_line(linewidth=2),
 independent_shapes=base+geom_point(aes(shape=a))+scale_shape_manual(values=c(A=21,B=24)),
 line_types=ggplot(d,aes(x,y,linetype=a))+geom_line()+scale_linetype_manual(values=c(A='dotted',B='longdash')),
 text_channels=ggplot(d,aes(x,y,label=a,angle=size,hjust=alpha,vjust=width))+geom_text(family='sans',fontface='bold',lineheight=1.2),
 after_scale_paints=base+geom_point(aes(fill=after_scale(colour)),shape=21),
 after_scale_alpha=base+geom_point(aes(alpha=after_scale(.5)),shape=21)
)
column <- function(x) lapply(seq_along(x),function(i) {v<-x[[i]];if(is.na(v)) return(list(number=if(is.nan(v)) 'NaN' else 'NA'));unname(v)})
cases <- lapply(names(plots),function(id) {
 warnings<-character();b<-withCallingHandlers(ggplot_build(plots[[id]]),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')})
 list(id=id,owner='GG-03',layers=lapply(b$data,function(d)list(row_count=nrow(d),columns=lapply(d,column))),
 scales=lapply(Filter(function(s)!any(s$aesthetics %in% c('x','y')),b$plot$scales$scales),function(s)list(aesthetics=as.list(s$aesthetics),domain=as.list(s$get_limits()))),warnings=as.list(warnings))
})
jsonlite::write_json(list(schema_version=1,reference='ggplot2 4.0.3',r_version=as.character(getRversion()),r_revision=R.version[['svn rev']],cases=cases,symbols=symbols),out,auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'aesthetic builds and',length(symbols),'native graphics-engine symbol PDFs\n')
