# GG14 grid distinguishes gpar fontsize big-points from physical TeX-point units.
library(grid);library(jsonlite)
stopifnot(getRversion()=="4.6.1")
pdf(NULL)
cases<-lapply(list(c(11,.9),c(16,1.4)),function(context){pushViewport(viewport(gp=gpar(fontsize=context[1],lineheight=context[2])));on.exit(popViewport());list(fontsize=context[1],lineheight=context[2],inches=setNames(lapply(c("points","bigpts","picas","dida","cicero","scaledpts","mm","cm","inches","lines","char"),function(u)convertUnit(unit(1,u),"inches",valueOnly=TRUE)),c("points","bigpts","picas","dida","cicero","scaledpts","mm","cm","inches","lines","char")))})
dev.off()
write_json(list(R=as.character(getRversion()),cases=cases),"fixtures/parity/ggplot2/theme-unit-context.json",auto_unbox=TRUE,digits=17,pretty=TRUE)
