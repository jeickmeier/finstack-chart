# GG07 source constructor and reversed-interval edge contracts.
library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
pdf(tempfile(fileext='.pdf'));cases=list();d=data.frame(x=1:3,y=1:3,v=c(1,1,2))
capture=function(name,p){b=ggplot_build(p);g=ggplotGrob(p);grid::grid.newpage();grid::grid.draw(g);cases[[length(cases)+1]]<<-list(name=name,data=b$data,draw_ok=TRUE)}
for(kind in c('hline','vline','abline')) {geom=switch(kind,hline=geom_hline(yintercept=0.5,alpha=0.5),vline=geom_vline(xintercept=0.5,alpha=0.5),abline=geom_abline(slope=1,intercept=0.5,alpha=0.5));capture(paste0('constant-',kind),ggplot(d,aes(x,y))+geom)}
capture('mapped-hline',ggplot(d,aes(x,y))+geom_hline(aes(yintercept=v),alpha=0.5))
for(kind in c('linerange','pointrange','errorbar','crossbar'))capture(paste0('reversed-',kind),ggplot(data.frame(x=1,y=1,lo=2,hi=0),aes(x,y,ymin=lo,ymax=hi))+get(paste0('geom_',kind))())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/reference-line-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null');dev.off();cat('Captured',length(cases),'reference edge cases\n')
