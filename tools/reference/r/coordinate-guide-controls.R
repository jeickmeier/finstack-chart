# Independent GG13 guide keys after source coordinate guide transformation.
library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
pdf(file=tempfile(fileext='.pdf'),width=8,height=6)
cases=list()
for(mode in c('full','partial','reverse','theta-y','secondary','angle','inside','at','outside-partial')) {
 co=switch(mode,partial=coord_radial(start=pi/4,end=pi*1.5,inner.radius=.3),reverse=coord_radial(reverse='thetar',inner.radius=.3),`theta-y`=coord_radial(theta='y',inner.radius=.3),inside=coord_radial(r.axis.inside=TRUE),at=coord_radial(r.axis.inside=c(1,3)),`outside-partial`=coord_radial(start=pi/4,end=3*pi/4,r.axis.inside=FALSE),coord_radial(inner.radius=.3))
 p=ggplot(data.frame(x=as.double(0:4),y=as.double(0:4)),aes(x,y))+geom_point()+scale_x_continuous(breaks=0:4,expand=c(0,0))+scale_y_continuous(breaks=0:4,expand=c(0,0))+co
 if(mode=='secondary')p=p+guides(theta.sec=guide_axis_theta(),r.sec=guide_axis())
 if(mode=='angle')p=p+guides(theta=guide_axis_theta(angle=0))
 b=ggplot_build(p);g=ggplot_gtable(b);pp=b$layout$panel_params[[1]]
 guides=lapply(c('theta','theta.sec','r','r.sec'),function(name){p=pp$guides$get_params(name);if(is.null(p))return(NULL);list(position=p$position,key=p$key,angle=if(is.numeric(p$angle))p$angle else NULL,label_angle_ccw=if(is.numeric(p$angle)&&"theta"%in%names(p$key))ggplot2:::flip_text_angle(p$angle-p$key$theta*180/pi)else NULL)})
 names(guides)=c('theta','theta.sec','r','r.sec')
 cases[[length(cases)+1]]=list(name=mode,theta=co$theta,arc=pp$arc,radii=pp$inner_radius,bbox=pp$bbox,theta_range=pp$theta.range,r_range=pp$r.range,axis_rotation=pp$axis_rotation,guides=guides)
}
facet_errors=list()
for(layout in c('wrap','grid'))for(free in c('free_x','free_y','free')){
  facet=if(layout=='wrap')facet_wrap(~g,scales=free)else facet_grid(.~g,scales=free)
  p=ggplot(data.frame(x=c(0,1,0,4),y=c(0,1,0,2),g=c('a','a','b','b')),aes(x,y))+geom_point()+facet+coord_fixed()
  result=tryCatch({ggplotGrob(p);list(ok=TRUE)},error=function(e)list(ok=FALSE,error=conditionMessage(e)))
  facet_errors[[length(facet_errors)+1]]=list(layout=layout,scales=free,result=result)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases,facet_errors=facet_errors),'fixtures/parity/ggplot2/coordinate-guide-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17,na='null',null='null')
dev.off()
