# FIX-GG04: ggplot point-size/stroke conversion, including zero-size observations.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3')
cases <- list()
dir.create('target/ggplot-point-sizes-reference',showWarnings=FALSE,recursive=TRUE)
for(shape in c(1,16,21)) for(stroke in c(0,.5,2)) {
  values <- c(-2,-1,-.1,0,.5,2,6)
  p <- ggplot2::ggplot(data.frame(x=seq_along(values),y=1,size=values),ggplot2::aes(x,y,size=size)) +
    ggplot2::geom_point(shape=shape,stroke=stroke) + ggplot2::scale_size_identity()
  b <- ggplot2::ggplot_build(p)
  d <- b$data[[1]]
  grob <- ggplot2:::GeomPoint$draw_panel(d,b$layout$panel_params[[1]],b$layout$coord)
  file <- sprintf('target/ggplot-point-sizes-reference/shape-%d-stroke-%s.pdf',shape,stroke)
  grDevices::pdf(file,compress=FALSE,width=4,height=2)
  grid::grid.draw(grob)
  grDevices::dev.off()
  pdf_lines <- readLines(file,warn=FALSE,skipNul=TRUE,encoding='latin1')
  widths <- as.numeric(sub(' w$','',grep('^[0-9.]+ w$',pdf_lines,value=TRUE,useBytes=TRUE)))
  cases[[length(cases)+1]] <- list(shape=shape,stroke=stroke,size=as.list(d$size),
    fontsize=as.list(grob$gp$fontsize),lwd=as.list(grob$gp$lwd),
    x=as.list(as.numeric(grob$x)),rows=nrow(d),pdf_widths=as.list(widths),pdf=file)
}
jsonlite::write_json(list(reference='ggplot2 4.0.3',cases=cases),
 'fixtures/parity/ggplot2/point-sizes.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(cases),'point-size configurations\n')
