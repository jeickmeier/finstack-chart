stopifnot(as.character(getRversion())=='4.6.1')
cases=list()
for(model in c('srgb','srgb+gray','rgb','rgb-nogray','gray','cmyk')){
 f=tempfile(fileext='.ps');postscript(f,width=2,height=1,horizontal=FALSE,paper='special',onefile=FALSE,colormodel=model,bg='transparent')
 grid::grid.rect(x=.25,width=.5,gp=grid::gpar(col=NA,fill='#804020'))
 grid::grid.rect(x=.75,width=.5,gp=grid::gpar(col=NA,fill='#808080'))
 dev.off();txt=readLines(f);start=grep('^%%Page:',txt);stopifnot(length(start)==1)
 cases[[length(cases)+1]]=list(model=model,paint_commands=as.list(txt[seq.int(start+1,length(txt))]));unlink(f)
}
jsonlite::write_json(list(R=as.character(getRversion()),cases=cases),'fixtures/parity/ggplot2/postscript-controls.json',pretty=TRUE,auto_unbox=TRUE,digits=17)
