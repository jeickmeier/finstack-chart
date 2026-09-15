pdf(file = tempfile(fileext = ".pdf"))
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
priority=lapply(0:20,function(n) list(count=n,order=unname(ggplot2:::axis_label_priority(n))))
print(ggplot2:::axis_label_priority_between)
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',priority=priority),'fixtures/parity/ggplot2/axis-guide-policies.json',auto_unbox=TRUE,pretty=TRUE,digits=17)
log_cases=list()
for(bounds in list(c(1,100),c(.001,10),c(-100,100),c(0,20),c(-10,-1),c(1e-10,1e10),c(1,1),c(.3,.7)))for(small in list(NULL,.01,.5)) {
 ticks=scales::minor_breaks_log(smallest=small)(bounds)
 log_cases[[length(log_cases)+1]]=list(bounds=bounds,smallest=small,values=unname(ticks),kind=unname(match(attr(ticks,'detail'),c(10,5,1))-1L))
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',priority=priority,log_ticks=log_cases),'fixtures/parity/ggplot2/axis-guide-policies.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null')
