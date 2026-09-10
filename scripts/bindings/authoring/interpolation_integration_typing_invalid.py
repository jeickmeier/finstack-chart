import finstack_chart as c
registry=c.ExtensionRegistry.example()
c.registered_interpolation(registry,'example.interpolation',1.5,{})
c.registered_interpolation(registry,'example.interpolation',1,lambda: None)
c.StandaloneScale('linear',registry='bad')
c.Interpolator.from_json('{}','bad')
c.piecewise(lambda a,b:c.interpolate_number(a,b),[0.,1.])
