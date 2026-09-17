# GG14 layout controls captured from the pinned source, independently of Rust.
library(ggplot2);library(grid);library(jsonlite)
stopifnot(as.character(packageVersion("ggplot2"))=="4.0.3")
d<-data.frame(x=1:4,y=c(2,1,4,3),g=c("A","A","B","B"))
base<-ggplot(d,aes(x,y,colour=g))+geom_point()+facet_wrap(vars(g),nrow=1)+theme_test()
extract<-function(p){g<-ggplotGrob(p);keep<-grepl("^(panel|axis|strip)",g$layout$name);list(layout=g$layout[keep,c("name","t","l","b","r")],widths=as.character(g$widths),heights=as.character(g$heights))}
cases<-list(inside=extract(base+theme(strip.placement="inside")),outside=extract(base+theme(strip.placement="outside",strip.switch.pad.wrap=unit(3,"mm"))),physical_panels=extract(base+theme(panel.widths=unit(c(2,4),"cm"))),weighted_panels=extract(base+theme(panel.widths=unit(c(1,3),"null"))))
write_json(list(reference="ggplot2 4.0.3",cases=cases),"fixtures/parity/ggplot2/theme-consumer-layout.json",auto_unbox=TRUE,digits=17,pretty=TRUE)
legend_case<-function(direction){g<-ggplotGrob(ggplot(d,aes(x,y,colour=g,size=x))+geom_point()+theme_test()+theme(legend.position="bottom",legend.box=direction,legend.spacing.x=unit(6,"mm"),legend.spacing.y=unit(4,"mm")));b<-g$grobs[[which(g$layout$name=="guide-box-bottom")]];list(layout=b$layout[,c("name","t","l","b","r")],widths=as.character(b$widths),heights=as.character(b$heights))}
current<-read_json("fixtures/parity/ggplot2/theme-consumer-layout.json",simplifyVector=FALSE)
current$legend_boxes<-list(horizontal=legend_case("horizontal"),vertical=legend_case("vertical"))
write_json(current,"fixtures/parity/ggplot2/theme-consumer-layout.json",auto_unbox=TRUE,digits=17,pretty=TRUE)
