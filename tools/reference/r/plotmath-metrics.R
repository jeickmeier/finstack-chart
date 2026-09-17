# Explicit-font device metrics are source evidence, separate from cross-host identity.
stopifnot(as.character(getRversion())=='4.6.1')
if(!isTRUE(capabilities('cairo')))stop('Explicit-font source metrics require the pinned Cairo/XQuartz oracle libraries.')
fonts=normalizePath('fixtures/math/fonts');config=tempfile(fileext='.conf')
writeLines(paste0('<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd"><fontconfig><dir>',fonts,'</dir><cachedir>',tempdir(),'</cachedir></fontconfig>'),config)
Sys.setenv(FONTCONFIG_FILE=config)
inventory=jsonlite::read_json('fixtures/parity/ggplot2/plotmath-syntax-inventory.json')
source=c(vapply(Filter(function(x)isTRUE(x$parseable),inventory$syntax),`[[`,'','syntax'),unlist(inventory$range_and_list_alias_expansion,use.names=FALSE))
cases=list();device=tempfile(fileext='.pdf')
withCallingHandlers(grDevices::cairo_pdf(device,width=8,height=6,family='DejaVu Serif',symbolfamily=grDevices::cairoSymbolFont('DejaVu Math TeX Gyre',usePUA=FALSE),pointsize=12),warning=function(w)stop(conditionMessage(w)))
stopifnot(names(dev.cur())=='cairo_pdf')
grid::grid.newpage()
for(size in c(12,24))for(text in source){
 result=tryCatch({expression=parse(text=text);g=grid::textGrob(expression,gp=grid::gpar(fontfamily='DejaVu Serif',fontsize=size));list(width=grid::convertWidth(grid::grobWidth(g),'inches',valueOnly=TRUE)*72,ascent=grid::convertHeight(grid::grobAscent(g),'inches',valueOnly=TRUE)*72,descent=grid::convertHeight(grid::grobDescent(g),'inches',valueOnly=TRUE)*72)},error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1]]=list(source=text,font_size=size,result=result)
}
metric_controls=list()
for(size in c(6,8.4,12,16.8,24,30))for(family in c('DejaVu Serif','DejaVu Math TeX Gyre'))for(text in c('x','y','z','M','X','+','0','g',',',' ', ', ', '\u2229','\u222a','\u2320','\u2321','\u22c5','\u00b0')){
 g=grid::textGrob(text,gp=grid::gpar(fontfamily=family,fontsize=size))
 metric_controls[[length(metric_controls)+1]]=list(text=text,font_family=family,font_size=size,width=grid::convertWidth(grid::grobWidth(g),'inches',valueOnly=TRUE)*72,ascent=grid::convertHeight(grid::grobAscent(g),'inches',valueOnly=TRUE)*72,descent=grid::convertHeight(grid::grobDescent(g),'inches',valueOnly=TRUE)*72)
}
dev.off()
number_sources=c('1e3','1e4','1e5','1e-3','1e-4','1.234567890123456','0001.00')
number_controls=lapply(number_sources,function(x)list(source=x,label=as.character(as.numeric(x))))
jsonlite::write_json(list(reference=list(R=as.character(getRversion()),device='cairo_pdf',font_family='DejaVu Serif',symbol_family='DejaVu Math TeX Gyre',symbol_use_PUA=FALSE,font_manifest=jsonlite::read_json('fixtures/math/fonts/manifest.json'),units='big points; measured via grid grob metrics'),cases=cases,metric_controls=metric_controls,number_controls=number_controls),'fixtures/parity/ggplot2/plotmath-metrics.json',pretty=TRUE,auto_unbox=TRUE,digits=17)
cat(length(cases),'fixed-font source metric cases\n')
