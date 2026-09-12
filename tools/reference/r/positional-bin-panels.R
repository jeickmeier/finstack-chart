# FIX-GG04: actual positional-bin charts and transformed panel ranges.
stopifnot(as.character(getRversion()) == '4.6.1', as.character(packageVersion('ggplot2')) == '4.0.3')
library(ggplot2)
encode <- function(v) lapply(v,function(x) if(is.nan(x)) 'NaN' else if(is.na(x)) NULL else if(is.infinite(x)) if(x>0) 'Infinity' else '-Infinity' else unname(x))
decode <- function(v) vapply(v,function(x)if(is.null(x))NA_real_ else if(is.character(x))switch(x,Infinity=Inf,`-Infinity`=-Inf,`NaN`=NaN)else x,numeric(1))
source <- jsonlite::read_json('fixtures/parity/ggplot2/positional-bins.json',simplifyVector=FALSE)$cases
cases <- list()
for(case in source) {
 if(!case$population %in% c('finite','constant')) next
 result <- tryCatch(suppressWarnings({
  mode <- case$mode
  s <- scale_x_binned(transform=case$transform,limits=if(is.null(case$limits))NULL else decode(case$limits),n.breaks=3,nice.breaks=mode!='equal',
   breaks=if(mode=='explicit')c(-1,1,2,4,10,20)else if(mode=='empty')numeric()else if(mode=='none')NULL else waiver(),right=case$right,show.limits=case$show_limits)
  x <- if(case$population=='constant')c(4,4)else c(1,10)
  p <- ggplot(data.frame(x=x,y=c(1,2)),aes(x,y))+geom_point()+s
  built <- ggplot_build(p);g <- ggplot_gtable(built);panel <- built$layout$panel_params[[1]]$x
  raw <- panel$scale$get_transformation()$inverse(panel$breaks)
  keep <- is.finite(panel$breaks)
  drawable <- keep & is.finite(panel$break_positions())
  coordinates <- built$layout$coord$transform(built$data[[1]],built$layout$panel_params[[1]])
  list(x=encode(built$data[[1]]$x),coordinate_x=encode(coordinates$x),
   drawable_break_values=encode(raw[drawable]),drawable_break_positions=encode(panel$break_positions()[drawable]),drawable_labels=as.list(unname(panel$get_labels()[drawable])),limits=encode(panel$limits),range=encode(panel$continuous_range),
   break_values=encode(raw[keep]),break_positions=encode(panel$break_positions()[keep]),labels=as.list(unname(panel$get_labels()[keep])))
 }),error=function(e)list(error=conditionMessage(e)))
 case$result <- result;cases[[length(cases)+1]] <- case
}
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',cases=cases),'fixtures/parity/ggplot2/positional-bin-panels.json',auto_unbox=TRUE,pretty=TRUE,digits=NA,null='null')
cat('PASS',length(cases),'actual positional-bin panels\n')
