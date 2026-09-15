# FIX-GG04: vector-coupled transform batches and their observable chart outcomes.
suppressPackageStartupMessages(library(ggplot2))
stopifnot(as.character(packageVersion('ggplot2')) == '4.0.3', as.character(packageVersion('scales')) == '1.4.0')
encode <- function(x) lapply(x, function(v) {
  if(is.na(v)) return(list(number = if(is.nan(v)) 'NaN' else 'NA'))
  if(is.infinite(v)) return(list(number = if(v > 0) 'Infinity' else '-Infinity'))
  v
})
capture <- function(value) tryCatch(value, error = function(e) list(error = conditionMessage(e)))
configurations <- list(); cases <- list(); batches <- list()
with_size <- Sys.getenv('VECTOR_SIZE') == '1'
with_facets <- Sys.getenv('VECTOR_FACETS') == '1'
with_layers <- Sys.getenv('VECTOR_LAYERS') == '1'
with_limits <- Sys.getenv('VECTOR_AUTHORED_LIMITS') == '1'
limit_options <- if(with_limits) list(c(0,20),c(20,0),c(NA,12),c(2,NA),c(-Inf,Inf)) else list(NULL)
pdf(tempfile('vector-transforms-', fileext='.pdf'))
for(family in c('cardinality', 'center')) for(composed in c(FALSE, TRUE)) {
  trace <- list()
  forward <- function(x) {
    trace[[length(trace)+1L]] <<- list(direction='forward', input=encode(x))
    if(family == 'cardinality') x + length(x) else x - mean(x, na.rm=TRUE)
  }
  inverse <- function(x) {
    trace[[length(trace)+1L]] <<- list(direction='inverse', input=encode(x))
    if(family == 'cardinality') x - length(x) else x
  }
  if(!composed) for(values in list(c(1,2,4,8,16), c(1,2,4,8,16,NA_real_,-Inf,Inf), numeric(), c(-Inf,Inf), c(1,NA_real_,3))) {
    batches[[length(batches)+1L]] <- list(family=family, input=encode(values), forward=encode(forward(values)), inverse=encode(inverse(values)))
  }
  tr <- scales::new_transform(family, forward, inverse)
  if(composed) tr <- capture(scales::transform_compose(tr, scales::transform_reverse()))
  configurations[[length(configurations)+1L]] <- list(family=family, composed=composed)
  for(route in if(with_size)'size'else c('position','paint','binned_paint')) for(population in c('ordinary','nonfinite','empty')) for(count in if(with_size)NA else c(NA,3)) for(authored_limits in limit_options) for(facet_policy in if(with_facets)c('fixed','free_x')else 'none') {
    trace <- list()
    x <- switch(population, ordinary=c(1,2,4,8,16), nonfinite=c(1,2,4,8,16,NA_real_,-Inf,Inf), empty=numeric())
    result <- if("error" %in% names(tr)) tr else capture(suppressWarnings({
      d <- data.frame(x=x, index=seq_along(x))
      if(with_facets) d$panel <- factor(rep(c('a','b'),length.out=length(x)), levels=c('a','b'))
      args <- list(transform=tr,n.breaks=if(is.na(count))NULL else count,limits=authored_limits)
      p <- if(route=='position') ggplot(d,aes(x,1))+geom_point()+do.call(scale_x_continuous,args) else ggplot(d,aes(index,1,colour=x))+geom_point()+do.call(if(route=='paint')scale_colour_continuous else scale_colour_steps,args)
      if(route=='size') p <- ggplot(d,aes(index,1,size=x))+geom_point()+do.call(scale_size_continuous,args[setdiff(names(args),'n.breaks')])
      second <- if(population=='empty') numeric() else c(2,10,32)
      if(with_layers) p <- p + geom_point(data=data.frame(x=second,index=seq_along(second)))
      if(with_facets) p <- p + facet_wrap(~panel,scales=facet_policy,drop=FALSE)
      b <- ggplot_build(p); build_calls <- trace
      grid::grid.draw(ggplotGrob(p))
      if(with_facets) {
        panels <- lapply(seq_along(b$layout$panel_params),function(i) {
          rows <- b$data[[1]][b$data[[1]]$PANEL==i,,drop=FALSE]
          panel <- b$layout$panel_params[[i]]$x
          list(mapped=if(route=='position')encode(rows$x)else if(route=='size')encode(rows$size)else as.list(rows$colour),range=encode(panel$continuous_range),
               breaks=capture(encode(panel$get_breaks())),positions=capture(encode(panel$break_positions())),
               labels=capture(as.list(panel$get_labels())),minor=capture(encode(panel$minor_breaks)))
        })
        list(build_draw='ok',panels=panels,build_calls=build_calls)
      } else if(route=='position') {
        panel <- b$layout$panel_params[[1]]$x
        list(build_draw='ok',mapped=encode(b$data[[1]]$x),mapped_second=if(with_layers)encode(b$data[[2]]$x)else NULL,range=encode(panel$continuous_range),
             breaks=capture(encode(panel$get_breaks())),positions=capture(encode(panel$break_positions())),
             labels=capture(as.list(panel$get_labels())),minor=capture(encode(panel$minor_breaks)),build_calls=build_calls)
      } else {
        scale <- b$plot$scales$get_scales('colour')
        list(build_draw='ok',mapped=as.list(b$data[[1]]$colour),mapped_second=if(with_layers)as.list(b$data[[2]]$colour)else NULL,limits=capture(encode(scale$get_limits())),
             breaks=capture(encode(scale$get_breaks())),labels=capture(as.list(scale$get_labels())),build_calls=build_calls)
      }
    }))
    cases[[length(cases)+1L]] <- list(configuration=length(configurations)-1L,route=route,population=population,
      facet=facet_policy,
      second=if(with_layers)encode(if(population=='empty')numeric()else c(2,10,32))else NULL,
      count=if(is.na(count))NULL else count,limits=if(is.null(authored_limits))NULL else encode(authored_limits),inputs=encode(x),result=result)
  }
}
invisible(dev.off())
jsonlite::write_json(list(reference='ggplot2 4.0.3 / scales 1.4.0 / R 4.6.1',
  configurations=configurations,cases=cases,batches=batches), if(with_size) 'fixtures/parity/ggplot2/vector-transform-size-facets.json' else if(with_facets && with_limits) 'fixtures/parity/ggplot2/vector-transform-facet-limits.json' else if(with_facets) 'fixtures/parity/ggplot2/vector-transform-facets.json' else if(with_layers && with_limits) 'fixtures/parity/ggplot2/vector-transform-layer-limits.json' else if(with_layers) 'fixtures/parity/ggplot2/vector-transform-layers.json' else if(with_limits) 'fixtures/parity/ggplot2/vector-transform-limits.json' else 'fixtures/parity/ggplot2/vector-transforms.json',
  auto_unbox=TRUE,pretty=TRUE,digits=17,null='null',na='null')
cat('Captured',length(cases),'vector transform charts; reference only.\n')
