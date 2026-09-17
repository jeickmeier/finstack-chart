# Independent GG14 oracle capture. Development-only ggplot2/R; no chart runtime is loaded.
library(ggplot2)
library(jsonlite)
library(grid)
stopifnot(as.character(packageVersion('ggplot2')) == '4.0.3', as.character(getRversion()) == '4.6.1')
encode <- function(x) {
  if (is.null(x)) return(list(kind='null'))
  if (inherits(x,'theme')) return(list(kind='theme',complete=x@complete,validate=x@validate,elements=lapply(as.list(x),encode)))
  if (inherits(x,'unit')) return(list(kind='unit',class=class(x),values=as.numeric(x),units=unitType(x)))
  if (inherits(x,'S7_object')) return(list(kind='element',class=class(x),properties=lapply(S7::props(x),encode)))
  if (is.function(x)) return(list(kind='function',formals=paste(deparse(formals(x)),collapse='\n')))
  if (is.language(x) || is.expression(x)) return(list(kind='expression',text=paste(deparse(x),collapse='\n')))
  if (is.list(x)) return(lapply(x,encode))
  list(kind=typeof(x),class=class(x),values=unname(unclass(x)))
}
observe <- function(code) {
  warnings<-character();messages<-character()
  result<-tryCatch(withCallingHandlers(list(value=encode(force(code))),warning=function(w){warnings<<-c(warnings,conditionMessage(w));invokeRestart('muffleWarning')},message=function(m){messages<<-c(messages,conditionMessage(m));invokeRestart('muffleMessage')}),error=function(e)list(error=conditionMessage(e)))
  c(result,list(warnings=unique(warnings),messages=unique(messages)))
}
tree<-get_element_tree();stopifnot(length(tree)==160L)
node<-function(x)list(class=if(inherits(x$class,'S7_class'))x$class@name else if(is.function(x$class))paste(deparse(x$class),collapse='\n')else as.character(x$class),parents=x$inherit,description=x$description)
resolve_all<-function(theme)setNames(lapply(names(tree),function(n)observe(calc_element(n,theme))),names(tree))
presets<-list();custom<-list(base_size=16,base_family='Fixture Body',header_family='Fixture Header',base_line_size=.7,base_rect_size=.9,ink='#123456',paper='#F8EEDD',accent='#D020A0')
for(name in c('grey','bw','linedraw','light','dark','minimal','classic','void','test')){
  fun<-get(paste0('theme_',name),asNamespace('ggplot2'))
  presets[[name]]<-list(default=list(authored=encode(fun()),resolved=resolve_all(fun())),custom=list(controls=custom,authored=encode(do.call(fun,custom)),resolved=resolve_all(do.call(fun,custom))))
}
stopifnot(identical(theme_grey(),theme_gray()))
cases<-list()
case<-function(name,patch,targets,base=theme_grey(),skip=FALSE){cases[[length(cases)+1]]<<-list(name=name,base=encode(base),patch=encode(patch),skip_blank=skip,resolved=setNames(lapply(targets,function(n)observe(calc_element(n,base+patch,skip_blank=skip))),targets))}
case('nested-relative',theme(text=element_text(size=20),axis.text=element_text(size=rel(.5)),axis.text.x=element_text(size=rel(.8))),c('text','axis.text','axis.text.x','axis.text.x.top'))
case('relative-linewidth',theme(line=element_line(linewidth=2),axis.line=element_line(linewidth=rel(.5)),axis.line.x.bottom=element_line(linewidth=rel(.25))),c('axis.line','axis.line.x.bottom'))
for(inherit in c(FALSE,TRUE))for(skip in c(FALSE,TRUE))case(paste('blank-parent',inherit,skip),theme(axis.text=element_blank(),axis.text.x=element_text(colour='red',inherit.blank=inherit)),c('axis.text.x','axis.text.x.bottom'),skip=skip)
case('blank-child',theme(axis.text.x=element_blank()),c('axis.text','axis.text.x','axis.text.x.bottom'))
case('partial-margin',theme(axis.title=element_text(margin=margin(1,2,3,4,'mm')),axis.title.x=element_text(margin=margin_part(t=7,l=9,unit='mm'))),c('axis.title','axis.title.x','axis.title.x.bottom'))
case('auto-margin',theme(plot.title=element_text(margin=margin_auto(3,5,unit='pt'))),c('plot.title'))
case('multi-parent-relative-length',theme(axis.ticks.length=unit(4,'mm'),axis.minor.ticks.length=rel(.5),axis.minor.ticks.length.x=rel(.25),axis.ticks.length.x.top=unit(8,'mm')),c('axis.minor.ticks.length.x.top','axis.minor.ticks.length.x.bottom'))
case('zero-negative-units',theme(axis.ticks.length=unit(-2,'mm'),panel.spacing=unit(0,'pt'),plot.margin=margin(-1,0,2,-3)),c('axis.ticks.length','panel.spacing','plot.margin'))
case('complete-theme-boundary',theme_void(),c('axis.text','axis.text.x','panel.grid','plot.title'),base=theme_bw())
case('missing-root',theme(text=element_text(size=rel(.5))),c('text','axis.text'),base=theme())
case('unknown-element',theme(),c('fixture.unknown'))
case('wrong-element-class',theme(axis.text=element_line(colour='red')),c('axis.text'))
# Context functions mutate only this short-lived R process. Preserve/restore the initial state.
initial<-get_theme()
context<-local({on.exit(set_theme(initial));set_theme(theme_grey());before<-get_theme();previous<-set_theme(theme_bw());set_return<-identical(previous,before)
  update_theme(axis.text=element_text(colour='red',size=14));updated<-get_theme();update_theme(axis.text=element_text(colour='blue'));merged<-get_theme();replace_theme(axis.text=element_text(colour='green'));replaced<-get_theme()
  list(set_returns_previous=set_return,updated=encode(updated),merged=encode(merged),replaced=encode(replaced),resolved=list(updated=observe(calc_element('axis.text',updated)),merged=observe(calc_element('axis.text',merged)),replaced=observe(calc_element('axis.text',replaced))))})
set_theme(initial)
subthemes<-list();for(name in sort(grep('^theme_sub_',getNamespaceExports('ggplot2'),value=TRUE))){fun<-get(name,asNamespace('ggplot2'));args<-if(grepl('axis',name))list(text=element_text(colour='red'),ticks.length=unit(3,'mm'))else switch(name,theme_sub_legend=list(text=element_text(size=rel(.8)),key.size=unit(5,'mm')),theme_sub_strip=list(background=element_rect(fill='pink'),text=element_text(angle=30)),theme_sub_plot=list(title=element_text(colour='blue'),margin=margin_auto(4)),theme_sub_panel=list(grid=element_line(colour='red'),spacing=unit(7,'mm')))
  subthemes[[name]]<-list(formals=encode(formals(fun)),empty=observe(fun()),authored=encode(args),expanded=observe(do.call(fun,args)),unknown=observe(fun(fixture_unknown=1)))}
a<-theme_grey()+theme(axis.text=element_text(colour='red'));b<-theme_bw()+theme(axis.text=element_text(colour='blue'));a2<-a+theme(axis.text=element_text(size=19));context$isolated<-list(a=encode(a),a_updated=encode(a2),b=encode(b),b_unchanged=identical(b,theme_bw()+theme(axis.text=element_text(colour='blue'))))
out<-list(reference=list(ggplot2=as.character(packageVersion('ggplot2')),R=as.character(getRversion())),element_tree=lapply(tree,node),presets=presets,grey_gray_identical=TRUE,inheritance=cases,contexts=context,subthemes=subthemes)
writeLines(toJSON(out,auto_unbox=TRUE,digits=17,na='null',pretty=TRUE),'fixtures/parity/ggplot2/theme-hierarchy-controls.json')
# Extract every documented syntax table row without interpreting/evaluating the expression.
rd<-tools::Rd_db('grDevices')[['plotmath.Rd']]
tag<-function(x)gsub('[^A-Za-z]','',attr(x,'Rd_tag') %||% '')
`%||%`<-function(a,b)if(is.null(a))b else a
plain<-function(x){if(!is.list(x))return(as.character(x));paste(vapply(x,plain,''),collapse='')}
tables<-list();walk<-function(x){if(tag(x)=='tabular'){
  rows<-list();cells<-character();text<-''
  for(item in x[[2]]) {t<-tag(item);if(t=='tab'){cells<-c(cells,trimws(text));text<-''}else if(t=='cr'){cells<-c(cells,trimws(text));if(any(nzchar(cells)))rows[[length(rows)+1]]<-cells;cells<-character();text<-''}else{text<-paste0(text,plain(item))}}
  if(nzchar(trimws(text)))rows[[length(rows)+1]]<-c(cells,trimws(text));tables[[length(tables)+1]]<<-rows
};if(is.list(x))invisible(lapply(x,walk))};walk(rd)
syntax<-list();for(t in seq_along(tables))for(r in seq_along(tables[[t]])){cells<-tables[[t]][[r]];if(length(cells)<2||cells[1]=='Syntax')next
  parsed<-tryCatch(list(parseable=TRUE,parsed=paste(deparse(parse(text=cells[1])),collapse='\n')),error=function(e)list(parseable=FALSE,error=conditionMessage(e)))
  syntax[[length(syntax)+1]]<-c(list(table=t,row=r,syntax=cells[1],description=cells[-1]),parsed)
}
stopifnot(length(tables)>0,length(syntax)>70)
greek<-c('alpha','beta','gamma','delta','epsilon','zeta','eta','theta','iota','kappa','lambda','mu','nu','xi','omicron','pi','rho','sigma','tau','upsilon','phi','chi','psi','omega')
aliases<-list(lowercase_greek=greek,uppercase_greek=paste0(toupper(substr(greek,1,1)),substring(greek,2)),variants=c('theta1','phi1','sigma1','omega1','Upsilon1'))

writeLines(toJSON(list(reference=list(R=as.character(getRversion()),package='grDevices',source='plotmath.Rd'),tables=tables,syntax=syntax,range_and_list_alias_expansion=aliases,scope='Complete documented syntax tables; parse-only, no expression evaluation or device metric claim.'),auto_unbox=TRUE,pretty=TRUE),'fixtures/parity/ggplot2/plotmath-syntax-inventory.json')
cat('PASS GG14 fixtures:',length(tree),'nodes,',length(presets),'presets x 2,',length(cases),'inheritance cases,',length(subthemes),'subthemes,',length(syntax),'syntax rows\n')

stopifnot(file.info('fixtures/parity/ggplot2/theme-hierarchy-controls.json')$size < 8*1024^2, file.info('fixtures/parity/ggplot2/plotmath-syntax-inventory.json')$size < 128*1024)
