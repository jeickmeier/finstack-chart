"""FIX-S08/09: registered shape protocols execute in shared Rust through actual Python."""
from pathlib import Path
import json, math, sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c

def op(name,parameters): return {'operation':{'id':'example.'+name,'version':'1'},'parameters':parameters}
def reject(fn,code=None):
    try: fn()
    except Exception as error:
        if code: assert error.code==code,(error,code)
    else: raise AssertionError('Expected rejection')
registry=c.ShapeRegistry.example()
shift=op('shift_curve',{'amount':2})
assert registry.selection(shift,'Curve')==shift
rows=[[0.,1.],[2.,3.],[9.,9.],[4.,2.],[5.,1.]]
config={'defined':[True,True,False,True,True]}
line=c.ShapeLine(config)
retained=line.generate_registered(rows,registry,shift)
assert retained.to_svg()=='M0,3L2,5M4,4L5,3'
for _ in range(3):
    area=c.ShapeArea(config).generate_registered(rows,registry,shift)
    assert area.to_svg()=='M0,3L2,5L2,2L0,2ZM4,4L5,3L5,2L4,2Z'
    area.dispose()
assert line.copy().config()==line.config()
assert c.ShapeLineRadial().generate_registered([[0.,2.],[math.pi/2,2.]],registry,shift).to_svg()=='M0,0L2,2'
radial=c.ShapeAreaRadial({'defined':config['defined']})
assert radial.generate_registered(rows,registry,op('shift_curve',{'amount':0})).to_svg()==radial.generate(rows).to_svg()
assert c.ShapeLink().generate_registered({'source':[0.,1.],'target':[2.,3.]},registry,shift).to_svg()=='M0,3L2,5'
assert c.ShapeSymbol({'size':16}).generate_registered(registry,op('rectangle_symbol',{'amount':4})).to_svg()=='M-4,-1h8v2h-8Z'
data=[{'rank':'9007199254740993','label':'a'},{'rank':'9007199254740992','label':'b'},{'rank':'9007199254740993','label':'c'}]
before=json.dumps(data)
comparator=op('field_comparator',{'field':'rank'})
pie=c.ShapePie({'angles':{'end_angle':6.}}).layout_registered(data,[1.,2.,3.],registry,comparator)
assert [p['index'] for p in pie]==[1,0,2]
assert [[p['start_angle'],p['end_angle']] for p in pie]==[[2.,3.],[0.,2.],[3.,6.]]
assert [p['data'] for p in pie]==data and json.dumps(data)==before
order=op('first_value_order',{});offset=op('shift_offset',{'amount':10})
stack=c.ShapeStack({'keys':['a','b']})
values=[[3.,1.],[4.,2.]]
result=stack.layout_registered(data[:2],values,registry,order,offset)
assert [s['index'] for s in result]==[1,0]
assert [[(p['y0'],p['y1']) for p in s['points']] for s in result]==[[(11.,14.),(12.,16.)],[(10.,11.),(10.,12.)]]
assert stack.layout_registered(data[:2],values,registry)==stack.layout(data[:2],values)
reject(lambda: registry.selection(op('native_curve',{'amount':2}),'Curve'),'CHART_UNSUPPORTED_CAPABILITY')
reject(lambda: registry.selection({**shift,'operation':{**shift['operation'],'version':'2'}},'Curve'),'CHART_UNSUPPORTED_CAPABILITY')
reject(lambda: line.generate_registered(rows,c.ShapeRegistry(),shift),'CHART_UNSUPPORTED_CAPABILITY')
reject(lambda: registry.selection(shift,'Symbol'),'CHART_VALIDATION')
reject(lambda: registry.selection(op('shift_curve',{'amount':2,'extra':1}),'Curve'),'CHART_VALIDATION')
reject(lambda: registry.selection(op('shift_curve',{'amount':'x'*65537}),'Curve'),'CHART_RESOURCE_LIMIT')
reject(lambda: registry.selection(op('shift_curve',{'amount':lambda: 2}),'Curve'))
reject(lambda: c.ShapeArea({**config,'limits':{'path':{**line.config()['limits']['path'],'max_commands':2}}}).generate_registered(rows,registry,shift),'CHART_RESOURCE_LIMIT')
reject(lambda: c.ShapePie().layout_registered([{},{}],[1.,1.],registry,comparator),'CHART_VALIDATION')
reject(lambda: c.ShapeStack({'keys':['a','b'],'limits':{'max_work':7}}).layout_registered(data[:2],values,registry,order,offset),'CHART_RESOURCE_LIMIT')
copy=registry.copy();registry.dispose()
reject(lambda: line.generate_registered(rows,registry,shift),'CHART_DISPOSED_HANDLE')
assert line.generate_registered(rows,copy,shift).to_svg()==retained.to_svg()
copy.dispose();assert retained.to_svg()=='M0,3L2,5M4,4L5,3'
print('PASS Python registered shapes: all five protocols; gaps, radial/links, exact source data, stack endpoints, deterministic reuse, explicit registration, portability, errors and budgets.')
