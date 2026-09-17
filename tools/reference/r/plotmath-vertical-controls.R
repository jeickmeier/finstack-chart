stopifnot(as.character(getRversion())=='4.6.1')
fonts=normalizePath('fixtures/math/fonts');config=tempfile(fileext='.conf')
writeLines(paste0('<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd"><fontconfig><dir>',fonts,'</dir><cachedir>',tempdir(),'</cachedir></fontconfig>'),config)
Sys.setenv(FONTCONFIG_FILE=config)
work=tempfile('math-vertical-');dir.create(work)
file.copy('tools/reference/r/plotmath-atom-metrics.c',file.path(work,'atom.c'))
old=getwd();setwd(work)
status=system2(file.path(R.home('bin'),'R'),c('CMD','SHLIB','atom.c'),stdout='build.log',stderr='build.log');if(status!=0)stop(paste(readLines('build.log'),collapse='\n'))
dyn.load(file.path(work,paste0('atom',.Platform$dynlib.ext)));setwd(old)
device=tempfile(fileext='.pdf');grDevices::cairo_pdf(device,width=8,height=6,family='DejaVu Serif',symbolfamily=grDevices::cairoSymbolFont('DejaVu Math TeX Gyre',usePUA=FALSE),pointsize=12)
rows=list()
for(size in c(12,24))for(spec in list(list(text='x',code=120,face=3),list(text='X',code=88,face=1),list(text='+',code=43,face=1),list(text='dotmath',code=-8901,face=5),list(text='integral-top',code=-8992,face=5),list(text='integral-bottom',code=-8993,face=5),list(text='a',code=97,face=3),list(text='b',code=98,face=3))){
 actual_size=if(spec$text %in% c('a','b'))size*.7 else size
 m=.Call('chart_atom_metric',as.integer(spec$code),as.integer(spec$face),actual_size)
 rows[[length(rows)+1]]=c(spec,list(base_size=size,font_size=actual_size,width=m[1],ascent=m[2],descent=m[3]))
}
dev.off()
jsonlite::write_json(list(reference=list(R=as.character(getRversion()),device='cairo_pdf',api='GEMetricInfo signed glyph metrics',font_manifest=jsonlite::read_json('fixtures/math/fonts/manifest.json')),atoms=rows),'fixtures/parity/ggplot2/plotmath-vertical-controls.json',pretty=TRUE,auto_unbox=TRUE,digits=17)
cat(length(rows),'signed glyph controls\n')
