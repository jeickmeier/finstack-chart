stopifnot(as.character(getRversion())=='4.6.1')
cases=list()
for(q in c(0,75,95,100)){
 f=tempfile(fileext='.jpeg');jpeg(f,width=16,height=16,quality=q,res=144,type='cairo');grid::grid.rect(gp=grid::gpar(fill='red',col=NA));dev.off()
 bytes=as.integer(readBin(f,'raw',n=file.info(f)$size));i=3;frame=NULL
 while(i<length(bytes)){
  stopifnot(bytes[i]==255);marker=bytes[i+1];n=bytes[i+2]*256+bytes[i+3]
  if(marker %in% c(192,193,194)){count=bytes[i+9];frame=list(marker=marker,width=bytes[i+7]*256+bytes[i+8],height=bytes[i+5]*256+bytes[i+6],sampling=as.list(bytes[i+11+seq.int(0,count-1)*3]));break}
  i=i+2+n
 }
 stopifnot(!is.null(frame));cases[[length(cases)+1]]=list(quality=q,result=frame);unlink(f)
}
jsonlite::write_json(list(R=as.character(getRversion()),device='jpeg(type=cairo)',cases=cases),'fixtures/parity/ggplot2/jpeg-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17)
