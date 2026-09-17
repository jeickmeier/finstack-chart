# One fresh-process mapproj evaluation. No prior initializer state is present.
library(mapproj)
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('mapproj'))=='1.2.12')
a=commandArgs(TRUE);v=jsonlite::read_json(a[1],simplifyVector=TRUE)
warnings=character();resolved=NULL
result=tryCatch(withCallingHandlers({out=mapproject(v$longitude,v$latitude,projection=v$method,parameters=v$parameters,orientation=v$orientation);resolved=get('.Last.projection',asNamespace('mapproj'))();out},warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')}),error=function(e)list(message=conditionMessage(e)))
if(!is.null(result$x)){classify=function(x)ifelse(is.nan(x),'NaN',ifelse(is.na(x),'NA',ifelse(is.infinite(x),ifelse(x>0,'+Inf','-Inf'),'finite')));result$x_class=classify(result$x);result$y_class=classify(result$y)}
v$result=result;v$resolved=resolved;v$warnings=warnings
jsonlite::write_json(v,a[2],auto_unbox=TRUE,digits=17,na='null',null='null')
