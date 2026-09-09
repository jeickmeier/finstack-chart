# Restore the accepted lock into the isolated workspace library, never the system library.
stopifnot(as.character(getRversion()) == "4.6.1")
renv::restore(project="tools/reference/r",library=Sys.getenv("R_LIBS_USER"),
              lockfile="tools/reference/r/renv.lock",prompt=FALSE)
