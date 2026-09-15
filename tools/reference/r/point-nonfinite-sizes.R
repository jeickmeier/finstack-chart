# FIX-GG04: reference mapped size values survive build, but nonfinite glyphs do not paint.
stopifnot(as.character(getRversion()) == '4.6.1',
          as.character(packageVersion('ggplot2')) == '4.0.3',
          as.character(packageVersion('ragg')) == '1.5.2')
library(ggplot2)
cases <- list()
for (shape in 0:25) for (size in c(-Inf, 1, Inf)) {
  capture <- ragg::agg_capture(width = 384, height = 288, res = 96, background = 'white')
  result <- tryCatch(suppressWarnings({
    p <- ggplot(data.frame(x = 1, y = 1, size = size), aes(x, y, size = size)) +
      geom_point(shape = shape) + scale_size_identity() + xlim(0, 2) + ylim(0, 2) + theme_void()
    b <- ggplot_build(p)
    print(p)
    pixels <- capture(native = FALSE)
    list(rows = nrow(b$data[[1]]),
         ink_pixels = sum(colSums(grDevices::col2rgb(as.vector(pixels)) != 255L) > 0L))
  }), error = function(e) list(error = conditionMessage(e)))
  invisible(dev.off())
  cases[[length(cases) + 1L]] <- list(shape = shape,
    size = if (is.finite(size)) size else if (size > 0) 'Infinity' else '-Infinity', result = result)
}
jsonlite::write_json(list(reference = 'ggplot2 4.0.3 / R 4.6.1 / ragg 1.5.2', cases = cases),
  'fixtures/parity/ggplot2/point-nonfinite-sizes.json',
  auto_unbox = TRUE, pretty = TRUE, digits = 15)
cat('captured', length(cases), 'point size raster outcomes\n')
