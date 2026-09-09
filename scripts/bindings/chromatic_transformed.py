"""Finite source values with IEEE exceptional log/power normalization."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
cases=json.loads((ROOT/'fixtures/parity/d3-scale-chromatic/transformed.json').read_text())['cases'];output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(320.,160.);errors=0
for case in cases:
    ramp=c.chromatic(case['ramp'],case['reverse']);scale=c.StandaloneScale(case['family'],**case['options'],interpolator=ramp);expected=case['expected']
    try:
        result=scale.map(case['input']);value=result.format_hex8();result.dispose()
    except c.ChartError:assert expected is None;errors+=1
    else:assert expected is not None and value=='#'+''.join(f'{n:02x}' for n in expected),case
    p=c.plot(c.Data.columns({'x':[0.],'value':[case['input']]})).aes(c.aes().x('x').y(1.).color('value').color_scale('named')).layer(c.points()).scale(c.color_mapped('named',scale)).build()
    request=output.request(p,options)
    try:frame=request.prepare()
    except c.ChartError:assert expected is None
    else:
        actual=[i['primitive']['Point']['fill'] for i in frame.scene()['items'] if 'Point'in i['primitive'] and i.get('layer') is not None]
        assert [[v['red'],v['green'],v['blue'],v['alpha']] for v in actual]==[expected],case;frame.dispose()
    for v in [request,p,scale,ramp]:v.dispose()
assert len(cases)==304 and errors==12
Path(sys.argv[2]).write_text(json.dumps({'cases':304,'expected_diagnostics':errors,'surfaces':['standalone','actual scene'],'exact':'RGBA'},indent=2)+'\n')
print('PASS FIX-21 Python: 304 exceptional normalization cases; valid reference colors and twelve explicit undefined-color diagnostics.')
