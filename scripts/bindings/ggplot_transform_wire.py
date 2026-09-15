"""FIX-GG04: standalone built-in transform envelopes through Python."""
import json
import math
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
records = []
registered='--registered' in sys.argv
registry=c.ExtensionRegistry.example() if registered else None
compositions='--compositions' in sys.argv
version=10 if registered else 9 if compositions else 8
transform={'Compose':{'transforms':['Asinh','Reverse']}} if compositions else 'Asinh'
if registered:transform={'Registered':{'selection':{'call':{'operation':{'id':'example.scale_transform','version':'1'},'parameters':{'family':'cubic','custom':False}}}}}
for kind in ('Numeric', 'Continuous', 'Sequential', 'Diverging', 'Ggplot'):
    base = c.StandaloneScale('linear' if kind in ('Numeric', 'Continuous') else 'diverging' if kind == 'Diverging' else 'sequential', **({'range': ['red', 'blue']} if kind == 'Continuous' else {}))
    spec = base.spec()
    if kind in ('Numeric', 'Continuous'):
        spec[kind]['family'] = {'Ggplot': {'transform': transform}}
    elif kind == 'Ggplot':
        spec['Interpolated']['normalization'] = {'Ggplot': {'family': {'Ggplot': {'transform': transform}}, 'domain': [0, 1], 'reverse': False, 'rescaler': 'Range'}}
    else:
        spec['Interpolated']['normalization'][kind]['family'] = {'Ggplot': {'transform': transform}}
    scale = c.StandaloneScale.from_spec(spec,registry)
    wire = scale.to_json(); envelope = json.loads(wire)
    assert envelope['version'] == version, envelope
    restored = c.StandaloneScale.from_json(wire,registry); copied = restored.copy()
    assert restored.to_json() == copied.to_json() == wire
    for value in (0., .25, .5, 1.):
        assert scale.map(value) == restored.map(value) == copied.map(value)
        if kind == 'Numeric':
            assert abs(scale.map(value) - (value**3 if registered else math.asinh(value) / math.asinh(1.))) < 2e-15
    for downgrade in range(1, version):
        envelope['version'] = downgrade
        try:
            invalid = c.StandaloneScale.from_json(json.dumps(envelope),registry)
        except Exception as error:
            assert 'version' in str(error).lower(), str(error)
        else:
            invalid.dispose()
            raise AssertionError((kind, version))
        assert scale.to_json() == wire
    if registered:
        try: c.StandaloneScale.from_json(wire)
        except Exception as error: assert 'register' in str(error).lower()
        else: raise AssertionError('Missing registry accepted')
        native_spec=json.loads(json.dumps(spec).replace('example.scale_transform','example.native_scale_transform'))
        native=c.StandaloneScale.from_spec(native_spec,registry)
        assert native.map(.5)==scale.map(.5)
        try:native.to_json()
        except c.ChartError as error:assert error.code=='CHART_UNSUPPORTED_CAPABILITY'
        else:raise AssertionError('Native transform serialized')
        native.dispose()
        wrong=json.loads(json.dumps(spec).replace('example.scale_transform','example.unknown_transform'))
        try:c.StandaloneScale.from_spec(wrong,registry)
        except c.ChartError as error:assert error.code=='CHART_UNSUPPORTED_CAPABILITY'
        else:raise AssertionError('Unknown transform installed')
        updated=scale.reconfigure(scale.spec()); assert updated.to_json()==wire;updated.dispose()
        configured=scale.configure(domain=list(scale.domain()));assert configured.map(.5)==scale.map(.5);configured.dispose()
    records.append({'kind': kind, 'envelope': json.loads(wire), 'downgrades_rejected': version - 1})
    for owned in (copied, restored, scale, base): owned.dispose()
legacy = c.StandaloneScale('linear')
assert json.loads(legacy.to_json())['version'] == 1
legacy.dispose()
if registry is not None:registry.dispose()
(out / 'states.json').write_text(json.dumps(records, indent=2) + '\n')
print(f'PASS Python transform wire: five containers, {5*(version-1)} rejected downgrades, copies/mapping and legacy v1 control.')
