# GG15 independent geography/CRS/label oracle, development-only sf binaries.
library(sf)
library(ggplot2)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('sf'))=='1.1.2',as.character(packageVersion('ggplot2'))=='4.0.3')
sf_use_s2(FALSE)
sq=function(x0,y0,x1,y1)rbind(c(x0,y0),c(x1,y0),c(x1,y1),c(x0,y1),c(x0,y0))
shapes=list(point=st_point(c(2,3)),multipoint=st_multipoint(rbind(c(0,0),c(2,0),c(4,6))),line=st_linestring(rbind(c(0,0),c(2,0),c(2,6))),multiline=st_multilinestring(list(rbind(c(0,0),c(4,0)),rbind(c(0,4),c(0,8)))),donut=st_polygon(list(sq(0,0,6,6),sq(2,2,4,4))),multipolygon=st_multipolygon(list(list(sq(0,0,1,1)),list(sq(10,0,14,4)))),collection=st_geometrycollection(list(st_point(c(20,20)),st_polygon(list(sq(0,0,4,4))))),empty=st_geometrycollection(),bowtie=st_polygon(list(rbind(c(0,0),c(2,2),c(0,2),c(2,0),c(0,0)))),hole_outside=st_polygon(list(sq(0,0,2,2),sq(3,3,4,4))))
loc=function(g,operation)tryCatch({v=operation(st_sfc(g));if(st_is_empty(v))NULL else unname(st_coordinates(v)[1,1:2])},error=function(e)list(error=conditionMessage(e)))
cases=lapply(names(shapes),function(name){g=shapes[[name]];list(name=name,wkt=st_as_text(g),valid=st_is_valid(st_sfc(g),reason=TRUE),centroid=loc(g,st_centroid),point_on_surface=loc(g,st_point_on_surface))})
crs_cases=list()
source=rbind(c(0,0),c(1,0),c(-73.5,45.5),c(30,60))
for(code in c(3857,32618,3035,4326)) {g=st_sfc(lapply(seq_len(nrow(source)),function(i)st_point(source[i,])),crs=4326);v=st_transform(g,code);crs_cases[[length(crs_cases)+1]]=list(epsg=code,proj=st_crs(code)$proj4string,input=unname(source),output=unname(st_coordinates(v)[,1:2]))}
d=st_sf(id=c('A','B'),geometry=st_sfc(shapes$donut,shapes$multipolygon,crs=4326))
coord_cases=list()
for(method in c('cross','box','orthogonal','geometry_bbox')) {
 p=ggplot(d)+geom_sf()+coord_sf(crs=3857,default_crs=4326,xlim=c(0,15),ylim=c(-2,8),lims_method=method)
 b=ggplot_build(p);q=b$layout$panel_params[[1]]
 coord_cases[[length(coord_cases)+1]]=list(method=method,x_range=q$x_range,y_range=q$y_range,graticule=lapply(st_drop_geometry(q$graticule),function(v)if(is.expression(v)||is.list(v))vapply(v,function(z)paste(deparse(z),collapse=''),character(1)) else v))
}
projected_labels=lapply(c('centroid','point_on_surface'),function(operation){g=st_sfc(st_polygon(list(sq(0,0,10,60))),crs=4326);fun=if(operation=='centroid')st_centroid else st_point_on_surface;list(operation=operation,input=unname(sq(0,0,10,60)),source_crs=4326,target_crs=3857,output=unname(st_coordinates(st_transform(fun(st_transform(g,3857)),4326))[1,1:2]))})
paint_cases=lapply(c('point','line','donut'),function(name){b=ggplot_build(ggplot(st_sf(geometry=st_sfc(shapes[[name]],crs=4326)))+geom_sf());data=b$data[[1]];list(name=name,style=as.list(data[1,intersect(names(data),c('shape','colour','fill','size','linewidth','linetype','alpha','stroke'))]))})
free_facet=tryCatch({ggplotGrob(ggplot(d)+geom_sf()+facet_wrap(~id,scales='free'));list(ok=TRUE)},error=function(e)list(error=conditionMessage(e)))
dateline=st_sfc(st_polygon(list(rbind(c(170,10),c(-170,10),c(-170,40),c(170,40),c(170,10)),rbind(c(160,20),c(-160,20),c(-160,30),c(160,30),c(160,20)))),crs=4326)
dateline_case=list(input=unclass(dateline[[1]]),projected=unclass(st_transform(dateline,3857)[[1]]),valid=st_is_valid(dateline,reason=TRUE))
map_coordinates=lapply(c('mercator','mollweide','tetra'),function(method){
  b=ggplot_build(ggplot(data.frame(x=c(-20,35),y=c(10,50)),aes(x,y))+geom_point()+coord_map(method,orientation=c(90,0,0)))
  q=b$layout$panel_params[[1]]
  list(method=method,x_range=q$x.range,y_range=q$y.range,x_projected=q$x.proj,y_projected=q$y.proj,x_breaks=q$x.major,y_breaks=q$y.major,x_labels=q$x.labels,y_labels=q$y.labels)
})
partial=st_sfc(st_point(c(0,100)),st_linestring(rbind(c(0,0),c(0,100),c(10,10))),st_polygon(list(rbind(c(0,0),c(0,100),c(10,10),c(0,0)))),st_multipoint(rbind(c(0,100),c(0,0))),crs=4326)
partial_transform=list(input=st_as_text(partial),output=st_as_text(st_transform(partial,3857)),coordinates=lapply(st_transform(partial,3857),function(g)if(st_is_empty(g))NULL else unname(st_coordinates(g)[,1:2,drop=FALSE])))
failed_limits_data=st_sf(geometry=st_sfc(st_polygon(list(sq(0,0,10,30))),crs=4326))
failed_limits_panel=suppressWarnings(ggplot_build(ggplot(failed_limits_data)+geom_sf()+coord_sf(crs=3857,default_crs=4326,xlim=c(0,10),ylim=c(100,110))))$layout$panel_params[[1]]
failed_limits=list(x_range=failed_limits_panel$x_range,y_range=failed_limits_panel$y_range)
out=list(failed_limits=failed_limits,partial_transform=partial_transform,map_coordinates=map_coordinates,dateline=dateline_case,free_facet=free_facet,paint=paint_cases,projected_labels=projected_labels,reference=list(R=as.character(getRversion()),sf=as.character(packageVersion('sf')),ggplot2=as.character(packageVersion('ggplot2')),external=as.list(sf_extSoftVersion()),s2=FALSE),labels=cases,crs=crs_cases,coordinates=coord_cases)
jsonlite::write_json(out,'fixtures/parity/ggplot2/geography-controls.json',pretty=TRUE,auto_unbox=TRUE,digits=17,na='null',null='null')
cat('Captured',length(cases),'geometry cases,',length(crs_cases),'CRS cases,',length(coord_cases),'limit policies\n')
