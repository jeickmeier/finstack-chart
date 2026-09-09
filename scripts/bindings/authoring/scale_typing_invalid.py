from finstack_chart import StandaloneScale, ScaleKey
StandaloneScale("unknown")
StandaloneScale("linear", domain=[object()])
StandaloneScale("utc", unit="fortnights")
StandaloneScale().floor("100", {"unit": "Hour", "step": 1})
StandaloneScale().configure(clamp="yes")
ScaleKey("Unsigned", "18446744073709551615")
