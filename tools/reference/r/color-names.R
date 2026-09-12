# FIX-GG04: complete R color catalog and parsing grammar, independent of CSS.
stopifnot(as.character(getRversion())=='4.6.1',as.character(packageVersion('ggplot2'))=='4.0.3')
names<-grDevices::colors()
parse<-function(input) tryCatch(list(rgba=as.list(as.integer(grDevices::col2rgb(input,alpha=TRUE)))),error=function(e)list(error=conditionMessage(e)))
catalog<-lapply(names,function(name)c(list(input=name),parse(name)))
inputs<-c('transparent','NA','na','Transparent','transparent ','red4','gray50','Dark Blue','RED','red ',
 ' red','dark  blue','dark\tblue','red\t','#ABC','#abcd','#0000','#ABC0','#12345678','#123456','#abcdef00',
 ' #123456','#12345','#ab','#123456789','rgb(1,2,3)','none','rebeccapurple','lightgoldenrodyellow',
 '1','2','8','9','0','-1','+1','1.5','1abc','01',' 1','1 ','1e2','99999999999999999999')
cases<-lapply(inputs,function(input)c(list(input=input),parse(input)))
jsonlite::write_json(list(reference='R 4.6.1 grDevices',catalog=catalog,cases=cases,
 palette=as.list(grDevices::palette())),'fixtures/parity/ggplot2/color-names.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS',length(catalog),'named colors and',length(cases),'parsing records\n')
manual_values<-c('gray','green','red4','gray50','Dark Blue','transparent','#ABC0','2')
d<-data.frame(x=seq_along(manual_values),y=1,group=letters[seq_along(manual_values)])
p<-ggplot2::ggplot(d,ggplot2::aes(x,y,colour=group))+ggplot2::geom_point()+
 ggplot2::scale_colour_manual(values=manual_values)
selected<-ggplot2::ggplot_build(p)$data[[1]]$colour
jsonlite::write_json(list(reference='ggplot2 4.0.3 / R 4.6.1',values=as.list(manual_values),
 rgba=lapply(selected,function(x)as.list(as.integer(grDevices::col2rgb(x,alpha=TRUE))))),
 'fixtures/parity/ggplot2/manual-color-text.json',auto_unbox=TRUE,pretty=TRUE,digits=NA)
cat('PASS eight manual-color text selections\n')
