# GG-00 executable oracle. Every record comes from pinned R/ggplot2, never Rust.
stopifnot(as.character(getRversion()) == "4.6.1", as.character(packageVersion("ggplot2")) == "4.0.3")
suppressPackageStartupMessages(library(ggplot2))
lock <- jsonlite::read_json("tools/reference/r/renv.lock",simplifyVector=FALSE)
for(package in lock$Packages) stopifnot(packageVersion(package$Package) == package_version(package$Version))
options(width = 120, error = function() { traceback(3); quit(status=1) })
Sys.setenv(TZ = "UTC")
RNGkind("Mersenne-Twister", "Inversion", "Rejection")
args <- commandArgs(trailingOnly = TRUE)
out <- if (length(args)) args[[1]] else "fixtures/parity/ggplot2"
dir.create(file.path(out,"artifacts"), recursive = TRUE, showWarnings = FALSE)
font <- normalizePath("fixtures/capability/fonts/NotoSans-Regular.ttf")
systemfonts::register_font("FinstackReference", plain = font)
theme_set(theme_gray(base_size = 11, base_family = "FinstackReference"))

# Explicit exceptional numeric values and ordered columns survive JSON without loss.
value <- function(x) {
  if (is.null(x)) return(NULL)
  if (is.expression(x) || is.language(x)) return(list(expression = paste(deparse(x),collapse="\n")))
  if (inherits(x,"sfc")) return(list(kind="sfc",crs=sf::st_crs(x)$wkt,geometry=lapply(unclass(x),value)))
  if (inherits(x,"sfg")) return(list(kind=class(x),coordinates=value(unclass(x))))
  if (is.matrix(x)) return(list(kind="matrix",dimensions=dim(x),values=value(as.vector(x))))
  if (inherits(x,"unit")) return(list(unit = unname(as.character(x))))
  if (is.factor(x)) return(list(kind = "factor", values = unname(as.character(x)), levels = levels(x), ordered = is.ordered(x)))
  if (is.data.frame(x)) return(list(row_count = nrow(x), columns = lapply(x,value)))
  if (is.list(x)) return(lapply(x,value))
  if (is.atomic(x)) {
    if (length(x) == 0) return(list())
    return(lapply(seq_along(x),function(i) {
      v <- x[[i]]
      if (is.numeric(v)) {
        if (is.nan(v)) return(list(number="NaN"))
        if (is.na(v)) return(list(number="NA"))
        if (!is.finite(v)) return(list(number=if(v>0) "Infinity" else "-Infinity"))
      }
      if (is.na(v)) return(list(missing=TRUE))
      unname(v)
    }))
  }
  list(kind = class(x)[[1]])
}
literal_title <- function(x) if(is.character(x)) x else value(x)
grob_record <- function(g) {
  fields <- intersect(names(g),c("label","x","y","x0","x1","y0","y1","width","height","r","id","id.lengths","pathId","hjust","vjust","rot","gp"))
  result <- list(kind = class(g), fields = lapply(unclass(g)[fields],value))
  children <- c(if(!is.null(g$grobs)) unname(g$grobs), if(!is.null(g$children)) unname(as.list(g$children)))
  if(length(children)) result$children <- lapply(children,grob_record)
  result
}
view_record <- function(v) {
  list(range = value(v$continuous_range), breaks = value(v$breaks),
       minor_breaks = value(v$minor_breaks), labels = value(v$get_labels()),
       limits = value(v$limits), discrete = v$is_discrete())
}
scales_record <- function(scales) lapply(scales, function(s) list(
  classes = class(s), aesthetics = s$aesthetics, range = value(s$range$range),
  limits = value(s$get_limits()), breaks = value(s$get_breaks()), labels = value(s$get_labels()),
  transform = if(!is.null(s$trans)) s$trans$name else NULL,
  palette = if(s$is_discrete()) value(s$map(s$get_limits())) else NULL))

d <- data.frame(x=c(0,1,2),y=c(1,3,2),category=factor(c("A","B","A"),levels=c("A","B")))
weighted <- data.frame(x=c(0,1,2),weight=c(1,2,3))
sample <- data.frame(x=seq_len(12),y=c(1,3,2,5,4,8,7,10,8,12,11,15), group=rep(c("A","B"),each=6))
grid <- expand.grid(x=c(-1,0,1),y=c(-1,0,1));grid$z <- grid$x+grid$y
square <- sf::st_sf(id=1L,geometry=sf::st_sfc(sf::st_polygon(list(matrix(c(-1,-1,1,-1,1,1,-1,1,-1,-1),ncol=2,byrow=TRUE))),crs=4326))
cases <- list(
  histogram_right = list(owner="GG-06",data=d,plot=ggplot(d,aes(x))+geom_histogram(breaks=c(0,1,2),closed="right")),
  histogram_left = list(owner="GG-06",data=d,plot=ggplot(d,aes(x))+geom_histogram(breaks=c(0,1,2),closed="left")),
  weighted_bins = list(owner="GG-06",data=weighted,plot=ggplot(weighted,aes(x,weight=weight))+geom_histogram(breaks=c(0,1,2))),
  inferred_groups = list(owner="GG-02",data=d,plot=ggplot(d,aes(x,y,colour=category))+geom_line()),
  explicit_all = list(owner="GG-02",data=d,plot=ggplot(d,aes(x,y,colour=category,group=1))+geom_line()),
  after_stat_count = list(owner="GG-02",data=d,plot=ggplot(d,aes(x=category,y=after_stat(count)))+geom_bar()),
  after_scale_colour = list(owner="GG-02",data=d,plot=ggplot(d,aes(x,y,colour=category,fill=after_scale(colour)))+geom_point(shape=21)),
  scale_filter = list(owner="GG-02",data=d,plot=ggplot(d,aes(x,y))+stat_summary(fun=mean,geom="point")+scale_x_continuous(limits=c(0,1))),
  coordinate_zoom = list(owner="GG-13",data=d,plot=ggplot(d,aes(x,y))+stat_summary(fun=mean,geom="point")+coord_cartesian(xlim=c(0,1))),
  manual_colour = list(owner="GG-04",data=d,plot=ggplot(d,aes(x,y,colour=category))+geom_point()+scale_colour_manual(values=c(A="#0050b4",B="#c81428"))),
  size_area = list(owner="GG-03",data=d,plot=ggplot(d,aes(x,y,size=x))+geom_point()+scale_size_area(max_size=6)),
  continuous_guide = list(owner="GG-05",data=d,plot=ggplot(d,aes(x,y,colour=x))+geom_point()),
  facets_free = list(owner="GG-12",data=d,plot=ggplot(d,aes(x,y))+geom_point()+facet_wrap(vars(category),scales="free")),
  facets_margins = list(owner="GG-12",data=d,plot=ggplot(d,aes(x,y))+geom_point()+facet_grid(rows=vars(category),margins=TRUE)),
  boxplot = list(owner="GG-09",data=sample,plot=ggplot(sample,aes(group,y))+geom_boxplot()),
  density = list(owner="GG-09",data=sample,plot=ggplot(sample,aes(y))+geom_density(n=64)),
  violin = list(owner="GG-09",data=sample,plot=ggplot(sample,aes(group,y))+geom_violin()),
  ecdf = list(owner="GG-09",data=sample,plot=ggplot(sample,aes(y))+stat_ecdf()),
  qq = list(owner="GG-09",data=sample,plot=ggplot(sample,aes(sample=y))+stat_qq()),
  lm = list(owner="GG-10",data=sample,plot=ggplot(sample,aes(x,y))+geom_smooth(method="lm",formula=y~x,n=12)),
  loess = list(owner="GG-10",data=sample,plot=ggplot(sample,aes(x,y))+geom_smooth(method="loess",formula=y~x,n=12)),
  quantile = list(owner="GG-10",data=sample,plot=ggplot(sample,aes(x,y))+geom_quantile(quantiles=c(.25,.5,.75))),
  hexbin = list(owner="GG-11",data=sample,plot=ggplot(sample,aes(x,y))+geom_hex(bins=3)),
  bin2d = list(owner="GG-11",data=sample,plot=ggplot(sample,aes(x,y))+geom_bin_2d(bins=3)),
  ellipse = list(owner="GG-11",data=sample,plot=ggplot(sample,aes(x,y))+stat_ellipse(type="norm",segments=20)),
  polar = list(owner="GG-13",data=d,plot=ggplot(d,aes(category,y,fill=category))+geom_col()+coord_polar()),
  labels = list(owner="GG-08",data=d,plot=ggplot(d,aes(x,y,label=category))+geom_label()),
  math_labels = list(owner="GG-14",data=d,plot=ggplot(d,aes(x,y))+geom_point()+labs(title=expression(alpha^2+sqrt(beta)))),
  contour = list(owner="GG-11",data=grid,plot=ggplot(grid,aes(x,y,z=z))+geom_contour(breaks=c(-1,0,1))),
  density2d = list(owner="GG-11",data=sample,plot=ggplot(sample,aes(x,y))+geom_density_2d(n=20)),
  function_curve = list(owner="GG-06",data=d,plot=ggplot(d,aes(x))+stat_function(fun=sin,n=11)),
  geography_sf = list(owner="GG-15",data=square,plot=ggplot(square)+geom_sf()+coord_sf(crs=3857))
)
results <- lapply(names(cases),function(id) {
  case <- cases[[id]]; set.seed(1729)
  warnings <- character()
  result <- withCallingHandlers({
    built <- ggplot_build(case$plot)
    base <- file.path(out,"artifacts",id)
    ragg::agg_png(paste0(base,".png"),width=500,height=300,res=100)
    grob <- ggplot_gtable(built)
    grid::grid.draw(grob); dev.off()
    svglite::svglite(paste0(base,".svg"),width=5,height=3); grid::grid.draw(grob); dev.off()
    grDevices::cairo_pdf(paste0(base,".pdf"),width=5,height=3,family=paste0("Noto Sans:file=",font)); grid::grid.draw(grob); dev.off()
    list(id=id,owner=case$owner,data=value(case$data),layers=lapply(built$data,value),
         panels=value(built$layout$layout), panel_parameters=lapply(built$layout$panel_params,function(p) {
           list(x=if(!is.null(p$x)) view_record(p$x) else NULL, y=if(!is.null(p$y)) view_record(p$y) else NULL,
                x_range=value(p$x.range),y_range=value(p$y.range))
         }), scales=scales_record(built$plot$scales$scales),
         guide_keys=lapply(unname(built$plot$guides$params),function(p) list(title=literal_title(p$title),key=value(p$key))),
         geometry=grob_record(grob))
  }, warning=function(w) { warnings <<- c(warnings,conditionMessage(w));invokeRestart("muffleWarning") })
  result$warnings <- warnings
  result
})
jsonlite::write_json(list(schema_version=1,reference="ggplot2 4.0.3",rng=list(seed=1729,kind=RNGkind()),
                         spatial_libraries=as.list(sf::sf_extSoftVersion()),
                         font=list(file="fixtures/capability/fonts/NotoSans-Regular.ttf",family="FinstackReference"), cases=results),
                     file.path(out,"cases.json"),auto_unbox=TRUE,pretty=TRUE,null="null",digits=NA)
cat("PASS GG-00:",length(results),"built-layer/scale/guide/panel/geometry records and",3*length(results),"reference artifacts.\n")
