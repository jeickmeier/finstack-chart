"""FIX-P01–06 through actual Python standalone handles and primary path annotations."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[3]);out.mkdir(parents=True,exist_ok=True)
rust=Path(sys.argv[2])
corpus=json.loads((ROOT/'fixtures/parity/d3-path/cases.json').read_text())
names={'moveTo':'move_to','lineTo':'line_to','quadraticCurveTo':'quadratic_curve_to','bezierCurveTo':'bezier_curve_to','arcTo':'arc_to','arc':'arc','rect':'rect','closePath':'close_path'}
def draw(p,op):
    args=op.get('args',[])
    if op['op']=='arc': args=[args[k] for k in ('x','y','r','a0','a1','anticlockwise')]
    return getattr(p,names[op['op']])(*args)
records=[]
for case in corpus['cases']:
    p=c.Path(case['digits']);observations=[];valid=True
    for operation in case['operations']:
        before=p.result();error=None
        try: draw(p,operation)
        except c.ChartError as e:
            error=e.code;valid=False;assert p.result()==before
        observations.append(dict(error=error,result=p.result()))
    records.append(dict(id=case['id'],observations=observations))
    saved=p.result();owned=p.copy();p.move_to(777,888);assert owned.result()==saved
    if valid:
        batch=c.Path(case['digits']).apply_batch(case['operations']);assert batch.result()==saved;batch.dispose()
        request=c.Path.from_json(json.dumps(dict(version=1,digits=case['digits'],operations=case['operations'])))
        assert request.result()==saved;request.dispose()
    owned.dispose();p.dispose()
(out/'paths.json').write_text(json.dumps(records,indent=2))
assert c.path_round().move_to(1.23456,-1.23456).to_svg()=='M1.235,-1.235'
for name,argc in [('move_to',2),('line_to',2),('quadratic_curve_to',4),('bezier_curve_to',6),('arc_to',5),('arc',5),('rect',4)]:
    for i in range(argc):
        for bad in [math.nan,math.inf,-math.inf]:
            p=c.path().move_to(0,0);before=p.result();args=[1.]*argc;args[i]=bad
            try: getattr(p,name)(*args)
            except c.ChartError as e: assert e.code=='CHART_NUMERICAL_DOMAIN'
            else: raise AssertionError('nonfinite accepted')
            assert p.result()==before;p.line_to(2,3);p.dispose()
for request in ['{"version":2,"operations":[]}','{"version":1,"operations":[{"op":"ellipse"}]}']:
    try: c.Path.from_json(request)
    except c.ChartError: pass
    else: raise AssertionError('invalid request accepted')
p=c.path().rect(0,0,10,10);before=p.result();accepted=[]
def sink(command):
    if len(accepted)==2:raise RuntimeError('stop')
    accepted.append(command)
try:p.replay(sink)
except RuntimeError:pass
assert len(accepted)==2 and p.result()==before
try:p.apply_batch([{'op':'moveTo','args':[20,20]},{'op':'arcTo','args':[0,0,1,1,-1]}])
except c.ChartError:pass
assert p.result()==before
p.dispose();p.dispose()
try:p.to_svg()
except c.ChartError as e:assert e.code=='CHART_DISPOSED_HANDLE'
else:raise AssertionError('disposed path readable')

plot=c.Plot.from_json((rust/'figure.plot.json').read_text());draft=plot.edit()
render=json.loads((ROOT/'fixtures/parity/d3-path/render.json').read_text())
def rebuild(draft):
    for case in render['cases']:
        p=c.path().apply_batch(case['operations'])
        component=c.vector_path(case['id'],p).anchor(case['anchor']).fill(case['fill']).stroke(case['stroke']).overflow(case['overflow'])
        if 'transform' in case: component=component.transform(case['transform'],max_error=0.001,max_commands=10000)
        p.move_to(999,999);p.dispose()
        draft=draft.annotation(component);component.dispose()
    return draft
draft=rebuild(draft)
updated=draft.build();output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
(out/'figure.plot.json').write_text(updated.to_json())
repeated_draft=rebuild(updated.edit());repeated=repeated_draft.build()
assert repeated.to_json()==updated.to_json(),'Rebuilding in the same runtime preserves exact definition identity'
repeated.dispose();repeated_draft.dispose()
for dpi in (300,600):
    options=c.export_options(render['width'],render['height']).dpi(dpi)
    request=output.request(updated,options);frame=request.prepare()
    scene=frame.scene();assert scene['version']==2
    assert all(scene['targets'][i]==[] for i,item in enumerate(scene['items']) if 'VectorPath' in item['primitive'])
    (out/f'figure-{dpi}.scene.json').write_text(json.dumps(scene,indent=2))
    for fmt in ('svg','pdf','png'):(out/f'figure-{dpi}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();options.dispose()
updated.dispose();draft.dispose();plot.dispose();output.dispose()
print('PASS FIX-P01–06 Python: 86 sequences, batches/copies/errors/disposal and six actual exports.')
