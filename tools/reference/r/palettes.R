# FIX-GG04: actual pinned palette outputs, never a local reimplementation.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3', as.character(packageVersion('scales')) == '1.4.0', as.character(packageVersion('viridisLite')) == '0.4.3')
records <- list()
record <- function(id, kind, args, expr) {
 warnings <- character()
 value <- tryCatch(withCallingHandlers(force(expr),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(error=conditionMessage(e)))
 records[[length(records)+1]] <<- list(id=id,kind=kind,args=args,value=as.list(value),warnings=as.list(warnings))
}
for (n in c(0,1,2,3,6,9,13,27)) for (direction in c(1,-1)) {
 a<-list(n=n,h=c(15,375),c=100,l=65,h_start=0,direction=direction)
 record(paste('hue',n,direction), 'hue',a,scales::pal_hue(direction=direction)(n))
 record(paste('grey',n,direction),'grey',list(n=n,start=if(direction==1).2 else .8,end=if(direction==1).8 else .2),scales::pal_grey(start=if(direction==1).2 else .8,end=if(direction==1).8 else .2)(n))
}
for(h in list(c(0,360),c(10,250),c(-100,720))) for(n in c(1,4,11)) {
 a<-list(n=n,h=h,c=65,l=40,h_start=33,direction=1)
 record(paste('hue',paste(h,collapse=':'),n),'hue',a,scales::pal_hue(h=h,c=65,l=40,h.start=33)(n))
}
for(name in rownames(RColorBrewer::brewer.pal.info)) for(n in c(0,1,2,3,5,9,12)) for(direction in c(1,-1)) {
 record(paste('brewer',name,n,direction),'brewer',list(name=name,n=n,direction=direction),scales::pal_brewer(palette=name,direction=direction)(n))
}
t <- c(-.1,0,.001,.1,.25,.5,.73,.9,.999,1,1.1,NA_real_,-Inf,Inf)
t_wire <- lapply(t,function(v)if(is.na(v))NULL else if(is.infinite(v))list(number=if(v>0)'Infinity' else '-Infinity') else v)
for(colours in list(c('#132B43','#56B1F7'),c('red','white','blue'),c('red'),c('transparent','red'),c('#00000000','#ff000080','#ffffffff'))) for(values in list(NULL,c(0,.2,1))) {
 if(!is.null(values)&&length(values)!=length(colours))next
 record(paste('gradient',paste(colours,collapse=':'),paste(values,collapse=':')),'gradient',list(colours=colours,values=values,t=t_wire),scales::pal_gradient_n(colours,values)(t))
}
for(values in list(c(0,.8,.2,1),c(0,.5,.5,1),c(.2,.3,.7,.9),c(0,0,1,1))) {
 colours<-c('red','yellow','cyan','blue')
 record(paste('gradient-extra',paste(values,collapse=':')),'gradient',list(colours=colours,values=values,t=t_wire),scales::pal_gradient_n(colours,values)(t))
}
for(option in LETTERS[1:8]) for(n in c(0,1,2,3,7,17,257)) for(direction in c(1,-1)) for(interval in list(c(0,1),c(.13,.79))) {
 record(paste('viridis',option,n,direction,paste(interval,collapse=':')),'viridis',list(option=option,n=n,direction=direction,begin=interval[1],end=interval[2],alpha=.7),viridisLite::viridis(n,alpha=.7,begin=interval[1],end=interval[2],direction=direction,option=option))
}
map <- viridisLite::viridis.map
seeds <- lapply(LETTERS[1:8],function(option){d<-map[map$opt==option,]; rgb<-grDevices::rgb(d$R,d$G,d$B); list(option=option,rgb=as.list(rgb),lab=unname(grDevices::convertColor(t(grDevices::col2rgb(rgb))/255,from='sRGB',to='Lab')))})
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / viridisLite 0.4.3 / R 4.6.1 r90187',cases=records,viridis_seeds=seeds),'fixtures/parity/ggplot2/palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,na='null',null='null')
cat('PASS',length(records),'palette cases\n')
