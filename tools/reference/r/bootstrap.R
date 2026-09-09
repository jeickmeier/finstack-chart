# Explicit setup only. Normal Rust tests never install or launch R.
stopifnot(as.character(getRversion()) == "4.6.1")
options(repos = c(CRAN = "https://cran.r-project.org"), timeout = 600)
library <- Sys.getenv("R_LIBS_USER")
dir.create(library, recursive = TRUE, showWarnings = FALSE)
packages <- c("ggplot2", "scales", "colorspace", "farver", "viridisLite",
              "RColorBrewer", "jsonlite", "renv", "quantreg", "hexbin", "maps",
              "mapproj", "sf", "svglite", "ragg", "Cairo")
missing <- packages[!packages %in% rownames(installed.packages(lib.loc = library))]
downloads <- "target/reference-r/downloads"
dir.create(downloads, recursive = TRUE, showWarnings = FALSE)
if (length(missing)) install.packages(missing, lib = library, type = "binary", destdir = downloads)
if (as.character(packageVersion("ggplot2")) != "4.0.3") {
  install.packages("https://cran.r-project.org/src/contrib/Archive/ggplot2/ggplot2_4.0.3.tar.gz",
                   lib = library, repos = NULL, type = "source")
}
stopifnot(as.character(packageVersion("ggplot2")) == "4.0.3")
cat("PASS isolated R 4.6.1 / ggplot2 4.0.3 reference installation\n")
print(sessionInfo())
