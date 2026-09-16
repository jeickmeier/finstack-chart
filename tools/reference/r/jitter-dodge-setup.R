stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
library(ggplot2)
cases=list()
for(x in list(c(1,3,5),c(1,1,3),c(2,2,2)))for(explicit in c(FALSE,TRUE)) {
 d=data.frame(x=x,y=1:3,group=1:3,PANEL=1L)
 p=position_jitterdodge(jitter.width=if(explicit).3 else NULL,seed=42)
 params=p$setup_params(d)
 cases[[length(cases)+1]]=list(x=x,explicit=explicit,width=params$jitter.width)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/jitter-dodge-setup.json',auto_unbox=TRUE,pretty=TRUE,digits=17)
