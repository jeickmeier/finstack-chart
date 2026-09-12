# FIX-GG03/GG04: physical linewidth conversion at the graphics-device boundary.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases<-list()
widths_of<-function(g){c(g$gp$lwd,unlist(lapply(g$children,widths_of),use.names=FALSE))}
dir.create('target/ggplot-line-widths-reference',recursive=TRUE,showWarnings=FALSE)
for(kind in c('segment','line','path','rect','polygon'))for(width in c(0,.2,.5,1,3,6)){
 d<-data.frame(x=c(1,2,3),y=c(1,2,1),linewidth=width)
 p<-ggplot(d,aes(x,y,linewidth=linewidth))+
 switch(kind,segment=geom_segment(aes(xend=x,yend=y+.5)),line=geom_line(),path=geom_path(),rect=geom_rect(aes(xmin=x-.2,xmax=x+.2,ymin=y-.2,ymax=y+.2),colour='black'),polygon=geom_polygon(colour='black'))+scale_linewidth_identity()
 b<-ggplot_build(p);layer<-b$plot$layers[[1]];grob<-layer$geom$draw_panel(b$data[[1]],b$layout$panel_params[[1]],b$layout$coord)
 file<-sprintf('target/ggplot-line-widths-reference/%s-%s.pdf',kind,width)
 grDevices::pdf(file,compress=FALSE,width=4,height=2);grid::grid.draw(grob);grDevices::dev.off()
 pdf_lines<-readLines(file,warn=FALSE,skipNul=TRUE,encoding='latin1');widths<-as.numeric(sub(' w$','',grep('^[0-9.]+ w$',pdf_lines,value=TRUE,useBytes=TRUE)))
 cases[[length(cases)+1]]<-list(kind=kind,linewidth=width,lwd=as.list(unname(widths_of(grob))),pdf_widths=as.list(widths))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/line-widths.json',auto_unbox=TRUE,pretty=TRUE,digits=17)
cat('PASS',length(cases),'physical linewidth reference builds\n')
