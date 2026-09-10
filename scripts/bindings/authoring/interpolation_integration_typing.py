import finstack_chart as c
registry = c.ExtensionRegistry.example()
factory: c.RegisteredInterpolationFactory = c.registered_interpolation(registry, 'example.interpolation', 1, {'mode':'SquaredNumber'})
value: c.Interpolator[c.InterpolationResult] = factory(0.,100.)
piece: c.Interpolator[c.InterpolationResult] = c.piecewise(factory,[0.,100.,200.])
scale = c.StandaloneScale('linear', registry=registry, factory=factory)
loaded = c.StandaloneScale.from_json(scale.to_json(), registry)
loaded_value = c.Interpolator.from_json(value.to_json(), registry)
with factory.copy() as owned:
    sample: c.InterpolationResult = owned(0.,1.)(0.5)
