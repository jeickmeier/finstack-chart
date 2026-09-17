library(ggplot2);library(grid);library(jsonlite)
stopifnot(as.character(packageVersion("ggplot2"))=="4.0.3")
g<-element_grob(element_line(colour="#123456",linewidth=.7,linetype="dashed",lineend="square",linejoin="bevel",arrow=arrow(angle=25,length=unit(4,"mm"),ends="both",type="closed"),arrow.fill="red"),x=c(0,1),y=c(0,0))
write_json(list(reference="ggplot2 4.0.3",gp=unclass(g$gp),arrow=list(angle=g$arrow$angle,length_mm=as.numeric(g$arrow$length),ends=g$arrow$ends,type=g$arrow$type)),"fixtures/parity/ggplot2/theme-line-controls.json",auto_unbox=TRUE,digits=17,pretty=TRUE)
