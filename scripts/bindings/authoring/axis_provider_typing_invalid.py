from finstack_chart import scale_registered, plot, Data
scale_registered('example.fold', '1', {})
scale_registered('example.fold', 1, lambda: 1)
plot(Data.columns({'x': [1.]})).with_registry('not a registry')
