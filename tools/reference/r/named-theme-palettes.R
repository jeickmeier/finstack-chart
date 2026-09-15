# FIX-GG04: the complete pinned named-palette registry and actual theme draws.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3',as.character(packageVersion('scales'))=='1.4.0')
library(ggplot2)
encode<-function(v)lapply(unname(v),function(x)if(is.na(x))NULL else if(is.infinite(x))if(x>0)'Infinity'else'-Infinity'else x)
names<-sort(scales:::palette_names());catalog<-list();brewer<-rownames(RColorBrewer::brewer.pal.info);viridis<-c('magma','inferno','plasma','viridis','cividis','rocket','mako','turbo');hcl<-grDevices::hcl.pals()
for(name in names){
 pal<-scales:::get_palette(name)
 entry<-if(name=='hue')list(kind='hue',levels=255)else if(name=='grey')list(kind='grey',levels=255)else if(name%in%viridis)list(kind='viridis',option=name,levels=255)else if(name%in%tolower(brewer))list(kind='brewer',id=brewer[match(name,tolower(brewer))],levels=scales:::palette_nlevels(pal))else if(inherits(pal,'pal_continuous'))list(kind='gradient',colors=encode(grDevices::hcl.colors(31,palette=hcl[match(name,tolower(hcl))])))else list(kind='manual',colors=encode(pal(scales:::palette_nlevels(pal))),levels=scales:::palette_nlevels(pal))
 catalog[[name]]<-entry
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',palettes=catalog),'fixtures/parity/ggplot2/named-palette-catalog.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
pdf(file=tempfile('named-palettes-',fileext='.pdf'));cases<-list()
for(name in c(names,'not-a-palette','ViRiDiS'))for(family in c('continuous','binned','discrete'))for(population in c('ordinary','missing','empty')){
 inputs<-if(family=='discrete')switch(population,ordinary=c('c','a','d','b'),missing=c(NA,'b','a',NA),empty=character())else switch(population,ordinary=c(-2,1,4,10,14),missing=c(NA,1,Inf,4,-Inf),empty=numeric())
 result<-tryCatch(suppressWarnings({
  scale<-do.call(switch(family,continuous=continuous_scale,binned=binned_scale,discrete=discrete_scale),list(aesthetics='colour',palette=NULL,guide='none',na.value=NA))
  args<-setNames(list(name),paste0('palette.colour.',if(family=='discrete')'discrete'else'continuous'))
  b<-ggplot_build(ggplot(data.frame(x=seq_along(inputs),v=inputs),aes(x,1,colour=v))+geom_point()+scale+do.call(theme,args));g<-ggplotGrob(b);grid::grid.draw(g)
  panel<-g$grobs[[which(g$layout$name=='panel')]];pts<-Filter(function(v)inherits(v,'points'),panel$children)
  list(mapped=encode(b$data[[1]]$colour),point_count=sum(vapply(pts,function(v)length(v$x),integer(1))),point_colours=encode(unlist(lapply(pts,function(v)v$gp$col),use.names=FALSE)))
 }),error=function(e)list(error=conditionMessage(e)))
 cases[[length(cases)+1L]]<-list(family=family,palette=name,population=population,na_mode='NA',inputs=encode(inputs),palette_values=list(name),result=result)
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/named-theme-palettes.json',auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('PASS',length(catalog),'registered palettes and',length(cases),'actual draws\n')
