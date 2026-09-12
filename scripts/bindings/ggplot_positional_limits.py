"""FIX-GG04 category continuous limits and ordering, independent of callbacks."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,360).dpi(96).basis('current');records=[]
fixture=json.loads((ROOT/'fixtures/parity/ggplot2/discrete-continuous-limits.json').read_text())
fixture['cases']+=json.loads((ROOT/'fixtures/parity/ggplot2/discrete-order.json').read_text())['cases']
def number(v):
 return {'number':'NaN'} if v is None else {'number':v} if isinstance(v,str) and v in ('Infinity','-Infinity','NaN') else v
for index,t in enumerate(fixture['cases']):
 for family in (['auto','band','point'] if t['levels'] is None else ['band','point']):
  for reverse in (False,True):
   owned=[];expected=t['result']
   try:
    n=len(t['inputs']);data=c.Data.columns({'x':c.categorical(t['inputs']),'y':c.column([1.]*n,kind='float64')},keys=list(range(100,100+n)));owned.append(data)
    axis=c.x_axis().range(540. if reverse else 100.,100. if reverse else 540.).guide_geometry({'labels':'Preserve'})
    if family!='auto':
     scale={'band':c.scale_band,'point':c.scale_point}[family]()
     if t['levels'] is not None:scale=scale.categories(t['levels'])
     axis=axis.scale(scale)
    expansion={'none':{'mult':[0.,0.],'add':[0.,0.]},'asymmetric':{'mult':[.1,.2],'add':[.3,.7]},'contract':{'mult':[-.5,-.25],'add':[-.2,-.1]}}.get(t['expansion'])
    limits=None if t['limits'] is None else [number(v) for v in t['limits']]
    axis=axis.expansion(expansion).continuous_limits(limits).minor_breaks({'Numeric':[-2,.5,1,1.5,2,3,4]})
    plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis).y_axis(c.y_axis().visible(False)).build();owned.append(plot)
    wire=plot.to_json();assert json.loads(wire)['version']==(19 if limits is None else 20)
    restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
    request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame);assert 'error' not in expected,(index,family,reverse)
    scene=frame.scene();points=[None]*n
    for item,targets in zip(scene['items'],scene['targets']):
     if not targets:continue
     assert len(targets)==1 and 'Source' in targets[0] and 'Point' in item['primitive']
     row=int(targets[0]['Source']['key'])-100;assert 0<=row<n and points[row] is None
     points[row]=(item['primitive']['Point']['center']['x']-100.)/440.
    for actual,wanted in zip(points,expected['point_positions']):
     if not isinstance(wanted,(int,float)):assert actual is None,(index,actual,wanted)
     else:assert actual is not None and math.isclose(actual,1-wanted if reverse else wanted,rel_tol=0,abs_tol=1e-12),(index,actual,wanted)
    guide=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom');ticks=guide['ticks'];minor=guide.get('minor_ticks',[])
    assert sorted(tick['label'] for tick in ticks)==sorted(expected['labels']),(index,ticks,expected)
    for label,position in zip(expected['labels'],expected['major_positions']):
     tick=next(tick for tick in ticks if tick['label']==label);assert math.isclose((tick['position']-100.)/440.,1-position if reverse else position,rel_tol=0,abs_tol=1e-12)
    assert len(minor)==len(expected['minor_values'])
    for i,tick in enumerate(minor):
     assert tick['value']['Number']==expected['minor_values'][i];position=expected['minor_positions'][i]
     assert math.isclose((tick['position']-100.)/440.,1-position if reverse else position,rel_tol=0,abs_tol=1e-12)
    records.append({'index':index,'family':family,'reversed':reverse,'points':points,'ticks':ticks,'minor_ticks':minor})
    sample=None
    if family=='band' and not reverse and t['expansion']=='default':
     if t['population']=='unused' and t['limits_name']=='wide':sample='unused-levels'
     elif t['population']=='three' and t['limits_name']=='right_inf':sample='one-infinite-endpoint'
     elif t['population']=='three' and t['limits_name']=='unbounded':sample='undefined-positions'
     elif t['population']=='all_outside' and t['limits_name']=='default':sample='excluded-population'
    if sample:
     (out/(sample+'.plot.json')).write_text(wire)
     for fmt in ('svg','pdf','png'):(out/(sample+'.'+fmt)).write_bytes(frame.export(fmt))
   except c.ChartError as error:
    assert 'error' in expected,(index,family,reverse,error);records.append({'index':index,'family':family,'reversed':reverse,'error':error.code})
   finally:
    for value in reversed(owned):value.dispose()
def update_data(labels):
    return c.Data.columns({'x':c.categorical(labels),'y':c.column([1.]*len(labels),kind='float64')},keys=list(range(100,100+len(labels))))
def update_plot(data,family,limits):
    axis=c.x_axis().scale({'band':c.scale_band,'point':c.scale_point}[family]().categories(['a','b','c','d'])).continuous_limits(limits).guide_geometry({'labels':'Preserve'})
    return c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis).y_axis(c.y_axis().visible(False)).build()
def selected(frame):
    scene=frame.scene();points=[item['primitive']['Point']['center'] for item,targets in zip(scene['items'],scene['targets']) if targets]
    guide=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')
    return {'points':points,'ticks':guide['ticks'],'minor_ticks':guide.get('minor_ticks',[])}
for family in ['band','point']:
    for limit_index,limits in enumerate([[1.5,2.5],[0.,{'number':'Infinity'}],[{'number':'-Infinity'},{'number':'Infinity'}]]):
        owned=[]
        try:
            original_data=update_data(['a','c']);owned.append(original_data)
            original=update_plot(original_data,family,limits);owned.append(original);wire=original.to_json()
            live=original.chart();owned.append(live)
            capture=output.request(live,options);owned.append(capture);held=capture.prepare();owned.append(held);initial=selected(held)
            for step,labels in enumerate([['d'],['z'],[],['a','c']]):
                step_owned=[]
                try:
                    replacement=update_data(labels);step_owned.append(replacement)
                    updates=live.transaction();step_owned.append(updates)
                    builder=updates.replace(original_data,replacement);step_owned.append(builder)
                    tx=builder.build();step_owned.append(tx);live.commit(tx)
                    request=output.request(live,options);step_owned.append(request);frame=request.prepare();step_owned.append(frame)
                    fresh_plot=update_plot(replacement,family,limits);step_owned.append(fresh_plot)
                    fresh_request=output.request(fresh_plot,options);step_owned.append(fresh_request);fresh=fresh_request.prepare();step_owned.append(fresh)
                    actual=selected(frame);assert actual==selected(fresh)
                    frozen=capture.prepare();step_owned.append(frozen);assert selected(frozen)==initial and selected(held)==initial
                    assert original.to_json()==wire
                    records.append({'kind':'replacement','family':family,'limit_index':limit_index,'step':step,**actual})
                finally:
                    for value in reversed(step_owned):value.dispose()
        finally:
            for value in reversed(owned):value.dispose()
output.dispose();assert len(records)==3582
(out/'records.json').write_text(json.dumps(records,indent=2))
print('PASS Python: 3558 positional limit/ordering configurations and 24 replacements.')
