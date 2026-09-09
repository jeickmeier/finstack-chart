"""FIX-S06/09 independent host scenes and publication artifacts at both resolutions."""
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
    elif type(actual) in (int,float) and type(expected) in (int,float) and re.fullmatch(r'/items/\d+/primitive/(?:ShapePath|VectorPath)/geometry/commands/\d+/(?:MoveTo/\d|LineTo/\d|QuadraticTo/\d|CubicTo/\d|Arc/(?:radius|to/\d))',path):
        difference=abs(actual-expected);assert difference<=2e-12*max(1,abs(expected)),(path,actual,expected);errors.append({'path':path,'absolute_error':difference})
    else:raise AssertionError((path,actual,expected))
for host in ['python','wasm']:
    for theme in ['Editorial','Terminal','Grayscale']:
        for dpi in [300,600]:
            name=f'figure-{dpi}';errors=[];base=root/'rust'/theme;actual=root/host/theme
            compare(json.loads((actual/f'{name}.scene.json').read_text()),json.loads((base/f'{name}.scene.json').read_text()),errors=errors)
            equal={ext:(base/f'{name}.{ext}').read_bytes()==(actual/f'{name}.{ext}').read_bytes() for ext in ['png','pdf','svg']}
            assert equal['png'] and equal['pdf'],(host,theme,dpi,equal)
            records.append(dict(host=host,theme=theme,dpi=dpi,coordinate_tolerance='2e-12 * max(1, abs(expected))',coordinate_differences=errors,other_scene_values_exact=True,publication_bytes_equal=equal))
(root/'comparison.json').write_text(json.dumps(records,indent=2)+'\n');print('PASS symbol comparison: three hosts, three themes, two resolutions, complete scene metadata and exact PNG/PDF bytes; SVG equality and any coordinate differences recorded. Visual/native inspection is separate.')
