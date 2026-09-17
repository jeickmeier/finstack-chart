# GG15 development-only oracle: full documented mapproj projection inventory.
library(mapproj)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('mapproj'))=='1.2.12',as.character(packageVersion('maps'))=='3.4.3')
methods=c('mercator','sinusoidal','cylequalarea','cylindrical','rectangular','gall','mollweide','gilbert','azequidistant','azequalarea','gnomonic','perspective','orthographic','stereographic','laue','fisheye','newyorker','conic','simpleconic','lambert','albers','bonne','polyconic','aitoff','lagrange','bicentric','elliptic','globular','vandergrinten','eisenlohr','guyou','square','tetra','hex','harrison','trapezoidal','lune','mecca','homing','sp_mercator','sp_albers')
parameters=list(cylequalarea=30,rectangular=30,gall=45,perspective=2,fisheye=1.5,newyorker=10,conic=30,simpleconic=c(20,50),lambert=c(20,50),albers=c(20,50),bonne=30,bicentric=30,elliptic=30,harrison=c(2,30),trapezoidal=c(20,50),lune=c(30,60),mecca=21,homing=21,sp_albers=c(20,50))
labels=list(cylequalarea='lat0',rectangular='lat0',gall='lat0',perspective='dist',fisheye='n',newyorker='r',conic='lat0',simpleconic=c('lat0','lat1'),lambert=c('lat0','lat1'),albers=c('lat0','lat1'),bonne='lat0',bicentric='lon0',elliptic='lon0',harrison=c('dist','angle'),trapezoidal=c('lat0','lat1'),lune=c('lat','angle'),mecca='lat0',homing='lat0',sp_albers=c('lat0','lat1'))
lon=c(0,30,-30,90,-90,179,-179,180,-180,0,0,0,0,180,NA,181)
lat=c(0,30,-30,45,-45,0,0,0,0,80,80.000001,90,-90,90,NA,91)
run=function(method,params,orientation,kind,x=lon,y=lat) {
 warnings=character();last=NULL
 output=tryCatch(withCallingHandlers({v=mapproject(x,y,projection=method,parameters=params,orientation=orientation);last=get('.Last.projection',asNamespace('mapproj'))();v},warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(message=conditionMessage(e)))
 if(!is.null(output$x)) { classify=function(v)ifelse(is.nan(v),'NaN',ifelse(is.na(v),'NA',ifelse(is.infinite(v),ifelse(v>0,'+Inf','-Inf'),'finite')));output$x_class=classify(output$x);output$y_class=classify(output$y) }
 list(method=method,case=kind,parameters=params,orientation=orientation,longitude=x,latitude=y,result=output,resolved=last,warnings=warnings)
}
cases=list();inventory=list()
for(method in methods) {
 p=parameters[[method]]
 inventory[[length(inventory)+1]]=list(method=method,parameter_names=if(is.null(labels[[method]]))character() else labels[[method]],parameter_count=length(p),anchor_parameters=p)
 cases[[length(cases)+1]]=run(method,p,c(90,0,0),'standard')
 cases[[length(cases)+1]]=run(method,p,NULL,'default_orientation',c(-100,-60,-20),c(-30,0,45))
 cases[[length(cases)+1]]=run(method,p,c(30,40,15),'tilted_orientation')
 cases[[length(cases)+1]]=run(method,p,c(90,180,-30),'rotated_dateline')
 cases[[length(cases)+1]]=run(method,NULL,c(90,0,0),'omitted_parameters')
 cases[[length(cases)+1]]=run(method,p,c(90,0),'invalid_orientation_length')
 if(length(p)>0) {
   for(value in c(0,-30,90,180))cases[[length(cases)+1]]=run(method,rep(value,length(p)),c(90,0,0),paste0('parameter_all_',value))
   if(length(p)==2)for(j in seq_along(p))for(value in c(0,-30,90,180)){v=p;v[j]=value;cases[[length(cases)+1]]=run(method,v,c(90,0,0),paste0('parameter_',j,'_',value))}
   cases[[length(cases)+1]]=run(method,c(p,1),c(90,0,0),'extra_parameter')
   if(length(p)==2)cases[[length(cases)+1]]=run(method,p[1],c(90,0,0),'missing_second_parameter')
 } else cases[[length(cases)+1]]=run(method,1,c(90,0,0),'unexpected_parameter')
}
# Stateful orientation reuse is observable; retain the exact resolved identity.
mapproject(c(-100,-20),c(0,30),projection='mercator')
reuse=run('',NULL,NULL,'reuse_previous_projection',c(-80,-40),c(0,30))
out=list(reference=list(R=as.character(getRversion()),mapproj=as.character(packageVersion('mapproj')),maps=as.character(packageVersion('maps'))),units=list(input='degrees',output='reference projection units; no assumed CRS meters'),inventory=inventory,cases=cases,reuse=reuse)
jsonlite::write_json(out,'fixtures/parity/ggplot2/mapproj-controls.json',pretty=TRUE,auto_unbox=TRUE,digits=17,na='null',null='null')
cat('Captured',length(methods),'methods and',length(cases),'independent calls\n')
