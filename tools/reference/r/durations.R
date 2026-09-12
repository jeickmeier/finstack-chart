# FIX-GG04: elapsed seconds are durations, not calendar instants.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3',
          as.character(packageVersion('scales')) == '1.4.0',
          as.character(packageVersion('hms')) == '1.1.4')
domains <- list(c(-1,1),c(0,0),c(0,.001),c(-.003,.007),c(0,120),
 c(0,120.01),c(0,7200),c(0,7200.01),c(0,172800),c(0,172800.01),
 c(0,1209600),c(0,1209600.01),c(-86400,86400),c(360001,720005),c(1e9,1e9+100))
cases <- list(); panels <- list(); widths <- list(); named <- list(); patterns <- list(); secondary <- list()
for (limits in domains) {
 for (n in c(3,5,8)) {
  values <- scales:::breaks_hms(n)(hms::as_hms(limits))
  cases[[length(cases)+1]] <- list(limits=as.list(limits),count=n,
   breaks=as.list(as.numeric(values)),exact=as.list(sprintf("%.17g",as.numeric(values))),labels=as.list(format(values)))
 }
 for (expand in c(FALSE,TRUE)) {
  p <- ggplot2::ggplot(data.frame(x=hms::as_hms(limits),y=c(0,1)),ggplot2::aes(x,y)) +
   ggplot2::geom_point() + ggplot2::scale_x_time(expand=ggplot2::expansion(if(expand).05 else 0))
  b<-ggplot2::ggplot_build(p); a<-b$layout$panel_params[[1]]$x; keep<-!is.na(a$breaks)
  panels[[length(panels)+1]] <- list(limits=as.list(limits),expand=expand,
   viewport=as.list(a$continuous_range),breaks=as.list(a$breaks[keep]),exact=as.list(sprintf("%.17g",a$breaks[keep])),labels=as.list(a$get_labels()[keep]))
 }
}
for (limits in list(c(-5.2,5.2),c(0,.02),c(0,172800),c(-7200,3600))) {
 for (width in if(diff(limits)<1)c(.001,.003,.007) else if(diff(limits)<20)c(.5,3,7) else c(3600,7000,86400)) {
  p <- ggplot2::ggplot(data.frame(x=hms::as_hms(limits),y=c(0,1)),ggplot2::aes(x,y)) +
   ggplot2::geom_point() + ggplot2::scale_x_time(expand=ggplot2::expansion(0),date_breaks=paste(width,'sec'))
  a<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x; keep<-!is.na(a$breaks)
  widths[[length(widths)+1]]<-list(limits=as.list(limits),seconds=width,
   breaks=as.list(a$breaks[keep]),exact=as.list(sprintf('%.17g',a$breaks[keep])),labels=as.list(a$get_labels()[keep]))
 }
}
for (unit in c('min','hour','day','week','month','year')) for (step in c(1,2)) {
 limits<-c(-4000000,40000000)
 p<-ggplot2::ggplot(data.frame(x=hms::as_hms(limits),y=c(0,1)),ggplot2::aes(x,y)) +
  ggplot2::geom_point() + ggplot2::scale_x_time(expand=ggplot2::expansion(0),date_breaks=paste(step,unit))
 # Keep fixture size bounded while crossing several origin-relative boundaries.
 if (unit=='min') limits<-c(-150,750)
 if (unit=='hour') limits<-c(-10000,50000)
 if (unit=='day') limits<-c(-100000,500000)
 if (unit=='week') limits<-c(-1000000,5000000)
 p$data$x<-hms::as_hms(limits)
 a<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x;keep<-!is.na(a$breaks)
 named[[length(named)+1]]<-list(limits=as.list(limits),unit=unit,step=step,
  breaks=as.list(a$breaks[keep]),labels=as.list(a$get_labels()[keep]))
}
for (limits in list(c(-7200,100000),c(-2,2),c(0,172800))) for(pattern in c('%H:%M:%S','%H:%M','%I:%M %p','%Y-%m-%d %H:%M:%S')) {
 p<-ggplot2::ggplot(data.frame(x=hms::as_hms(limits),y=c(0,1)),ggplot2::aes(x,y))+
  ggplot2::geom_point()+ggplot2::scale_x_time(expand=ggplot2::expansion(0),date_labels=pattern)
 a<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x;keep<-!is.na(a$breaks)
 patterns[[length(patterns)+1]]<-list(limits=as.list(limits),pattern=pattern,
  breaks=as.list(a$breaks[keep]),labels=as.list(a$get_labels()[keep]))
}
for(limits in list(c(-100,500),c(0,86400),c(0,0))) for(affine in list(c(1,3600),c(2,0),c(-1,7200))) {
 factor<-affine[1];offset<-affine[2]
 p<-ggplot2::ggplot(data.frame(x=hms::as_hms(limits),y=c(0,1)),ggplot2::aes(x,y))+
  ggplot2::geom_point()+ggplot2::scale_x_time(sec.axis=ggplot2::sec_axis(function(x)x*factor+offset))
 info<-ggplot2::ggplot_build(p)$layout$panel_params[[1]]$x.sec$break_info;order<-order(info$major)
 secondary[[length(secondary)+1]]<-list(limits=as.list(limits),factor=factor,offset=offset,
  range=as.list(as.numeric(info$range)),values=as.list(as.numeric(info$major_source_user[order])),
  labels=as.list(info$labels[order]),positions=as.list(as.numeric(info$major[order])))
}
label_inputs <- list(c(-1,0,1),c(0,.0000001,.1234567),c(0,360000,1),c(.1,.01,1),
 c(-.0000006,.0000004,.9999996),c(-360001.123456,0,36000.1),c(59.9999996,60,3599.9999996))
labels<-lapply(label_inputs,function(x)list(values=as.list(x),exact=as.list(sprintf("%.17g",x)),labels=as.list(format(hms::as_hms(x)))))
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / hms 1.1.4',
 cases=cases,panels=panels,labels=labels,widths=widths,named=named,patterns=patterns,secondary=secondary),'fixtures/parity/ggplot2/durations.json',
 auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'duration break records,',length(panels)+length(widths)+length(named)+length(patterns),'primary panels,',length(secondary),'secondary panels,',length(labels),'label vectors\n')
