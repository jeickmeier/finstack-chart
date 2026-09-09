"""FIX-S04: exact metadata/publication; declared numeric tolerance for arc coordinates."""
from pathlib import Path
import json,re,sys
root=Path(sys.argv[1]);records=[]
def compare(actual,expected,path='',errors=None):
    if actual==expected:return
    if isinstance(actual,dict) and isinstance(expected,dict):
        assert actual.keys()==expected.keys(),path
        for k in actual:compare(actual[k],expected[k],path+'/'+k,errors)
    elif isinstance(actual,list) and isinstance(expected,list):
        assert len(actual)==len(expected),path
        for i,(a,b) in enumerate(zip(actual,expected)):compare(a,b,path+'/'+str(i),errors)
    elif type(actual) in (int,float) and type(expected) in (int,float) and re.fullmatch(r'/items/\d+/primitive/ShapePath/geometry/commands/\d+/(?:MoveTo/\d|LineTo/\d|QuadraticTo/\d|CubicTo/\d|Arc/(?:radius|to/\d))',path):
        difference=abs(actual-expected);assert difference<=2e-12*max(1,abs(expected)),(path,actual,expected)
        errors.append({'path':path,'absolute_error':difference})
    else:raise AssertionError((path,actual,expected))
for host in ['python','wasm']:
    for dpi in [300,600]:
        name=f'figure-{dpi}';errors=[]
        compare(json.loads((root/host/f'{name}.scene.json').read_text()),json.loads((root/'rust'/f'{name}.scene.json').read_text()),errors=errors)
        assert (root/'rust'/f'{name}.png').read_bytes()==(root/host/f'{name}.png').read_bytes(),(host,dpi,'png')
        records.append({'host':host,'dpi':dpi,'coordinate_tolerance':'2e-12 * max(1, abs(expected))','coordinate_differences':errors,'all_other_scene_values_exact':True,'png_bytes_exact':True})
(root/'comparison.json').write_text(json.dumps(records,indent=2)+'\n')
print('PASS FIX-S04: exact source/scene metadata and PNG bytes at 300/600 DPI; raw arc coordinates meet the predeclared numeric tolerance. Native/SVG/PDF inspection is separate.')
