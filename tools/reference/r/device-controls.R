# Offline GG17 device and explicit saving policies; private oracle only.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
formals_list=function(f)lapply(formals(f),function(x)paste(deparse(x),collapse=' '))
namespace=asNamespace('ggplot2')
cases=list()
for(unit in c('in','cm','mm','px'))for(dpi in list(72,144,'screen','print','retina')){
 args=list(width=10,height=5,scale=1,units=unit,dpi=dpi,limitsize=TRUE)
 result=tryCatch({resolution=get('parse_dpi',namespace)(dpi);get('plot_dim',namespace)(dim=c(10,5),scale=1,units=unit,limitsize=TRUE,dpi=resolution)},error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]=list(input=args,result=result)
}
for(size in c(49.99,50,50.01))for(limit in c(TRUE,FALSE))cases[[length(cases)+1]]=list(input=list(width=size,height=1,units='in',limitsize=limit),result=tryCatch(get('plot_dim',namespace)(dim=c(size,1),scale=1,units='in',limitsize=limit,dpi=300),error=function(e)list(error=conditionMessage(e))))
devices=lapply(c('jpeg','tiff','bmp','postscript','pictex','svg','pdf','png'),function(name)list(name=name,formals=formals_list(get(name,asNamespace('grDevices')))))
jsonlite::write_json(list(reference=list(R=as.character(getRversion()),ggplot2=as.character(packageVersion('ggplot2')),platform=R.version$platform,capabilities=as.list(capabilities())),devices=devices,save_formals=formals_list(ggplot2::ggsave),dimensions=cases),'fixtures/parity/ggplot2/device-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17)
cat(length(cases),'dimension cases and',length(devices),'device signatures\n')
fixture=jsonlite::read_json('fixtures/parity/ggplot2/device-controls.json',simplifyVector=FALSE)
codec_cases=list()
for(type in c('cairo','quartz'))for(compression in c('none','rle','lzw','jpeg','zip','lzw+p','zip+p','lerc','lzma','zstd','webp')){
 file=tempfile(fileext='.tiff');warnings=character()
 result=tryCatch(withCallingHandlers({grDevices::tiff(file,width=16,height=16,res=72,type=type,compression=compression);grid::grid.rect(gp=grid::gpar(fill='red',col=NA));grDevices::dev.off();list(output_created=file.exists(file),bytes=unname(file.info(file)$size))},warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e){if(dev.cur()>1)dev.off();list(error=conditionMessage(e))})
 codec_cases[[length(codec_cases)+1]]=list(type=type,compression=compression,result=result,warnings=as.list(warnings))
 unlink(file)
}
fixture$tiff_compression_cases=codec_cases
jsonlite::write_json(fixture,'fixtures/parity/ggplot2/device-controls.json',auto_unbox=TRUE,pretty=TRUE,digits=17)
cat(length(codec_cases),'actual TIFF source device calls\n')
