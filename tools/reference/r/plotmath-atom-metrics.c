/* Oracle-only bridge to the pinned device's public glyph metrics interface. */
#include <R.h>
#include <Rinternals.h>
#include <R_ext/GraphicsEngine.h>
#include <R_ext/GraphicsDevice.h>
#include <string.h>
SEXP chart_atom_metric(SEXP code, SEXP face, SEXP size) {
    R_GE_gcontext gc;
    memset(&gc, 0, sizeof(gc));
    gc.cex = 1;
    gc.ps = asReal(size);
    gc.fontface = asInteger(face);
    strcpy(gc.fontfamily, "DejaVu Serif");
    pGEDevDesc dd = GEcurrentDevice();
    double ascent, descent, width;
    GEMetricInfo(asInteger(code), &gc, &ascent, &descent, &width, dd);
    SEXP result = PROTECT(allocVector(REALSXP, 3));
    REAL(result)[0] = GEfromDeviceWidth(width, GE_INCHES, dd) * 72;
    REAL(result)[1] = GEfromDeviceHeight(ascent, GE_INCHES, dd) * 72;
    REAL(result)[2] = GEfromDeviceHeight(descent, GE_INCHES, dd) * 72;
    UNPROTECT(1);
    return result;
}
