from finstack_chart import Data, plot, labels, title, subtitle, legend

data = Data.columns({'x':[1,2,3]})
labels().title('Wrong')
plot(data).title(labels())
plot(data).title(subtitle('Wrong'))
plot(data).layer(legend())
plot(data).title(title('Okay')).build().edit().data(data)
