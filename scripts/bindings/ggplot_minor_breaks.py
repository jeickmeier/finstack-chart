"""FIX-GG04 actual numeric minor selection through Python and immutable guide snapshots."""
import json
import math
import sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(640,360).basis('current');records=[]
fixture=json.loads((ROOT/'fixtures/parity/ggplot2/minor-breaks.json').read_text())
for index,case in enumerate(fixture['cases']):
    owned=[]
    try:
        values={'finite':[1.,10.],'constant':[4.,4.],'negative':[-4.,10.],'zero_wide':[0.,10.]}[case['population']]
        data=c.Data.columns({'x':values,'y':[1.,2.]});owned.append(data)
        scale={'identity':c.scale_linear,'sqrt':c.scale_sqrt,'reverse':c.scale_reverse,'log10':lambda:c.scale_log(10.)}[case['transform']]()
        axis=c.x_axis().scale(scale).range(0.,100.)
        if case['population']=='zero_wide':axis=axis.expansion({'mult':[1.,1.],'add':[0.,0.]})
        major={'regular':[1,2,4,10],'descending':[10,4,2,1],'outside':[-5,0,2,20],'one':[4],'empty':[],'none':[]}.get(case['major'])
        if major is not None: axis=axis.tick_values(major)
        minor={'auto':'Automatic','none':'Hidden','empty':{'Numeric':[]},'explicit':{'Numeric':[{'number':'-Infinity'},-2,0,.5,1,3,6,10,12,{'number':'Infinity'},{'number':'NaN'}]}}[case['minor']]
        plot=(c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis.minor_breaks(minor)).build());owned.append(plot)
        wire=plot.to_json();assert json.loads(wire)['version']==19
        restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
        request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
        guide=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')
        ticks=guide.get('minor_ticks',[]);expected=case['result'];assert 'error' not in expected
        assert len(ticks)==len(expected['minor_values']),(index,ticks,expected)
        for i,tick in enumerate(ticks):
            if expected['minor_values'][i] is None: assert tick['value'] is None
            else: assert math.isclose(tick['value']['Number'],expected['minor_values'][i],rel_tol=1e-11,abs_tol=1e-11),(index,tick,expected)
            assert math.isclose(tick['position']/100.,expected['minor_positions'][i],rel_tol=1e-11,abs_tol=1e-11),(index,tick,expected)
        records.append({'index':index,'minor_ticks':ticks})
    finally:
        for value in reversed(owned):value.dispose()
temporal_options=c.export_options(640,360).basis('current').layout(c.layout_options().max_ticks(4096))
temporal_cases=[case for name in ('minor-time-widths.json','minor-time-auto.json','minor-time-values.json') for case in json.loads((ROOT/'fixtures/parity/ggplot2'/name).read_text())['cases']]
for index,case in enumerate(temporal_cases):
    owned=[]
    try:
        kind=case['kind']; multiplier=86400000000 if kind=='date' else 1000000
        values=case['inputs'] if kind=='duration' else c.timestamps([round(v*multiplier) for v in case['inputs']],'us','UTC')
        data=c.Data.columns({'x':values,'y':[1.,2.]});owned.append(data)
        scale={'date':c.scale_date,'datetime':c.scale_utc,'duration':c.scale_duration}[kind]()
        policy={'TimeWidth':case['width']} if 'width' in case else 'Automatic'
        if 'minor_values' in case:policy={'Timestamps':[None if v is None else {'Timestamp':{'value':str(round(v*multiplier)),'unit':'Microseconds'}} for v in case['minor_values']]}
        axis=c.x_axis().scale(scale).range(0.,100.).minor_breaks(policy)
        if 'major' in case:
            ratios={'regular':[0,.25,.75,1],'descending':[1,.75,.25,0],'one':[0],'empty':[]}.get(case['major'])
            if ratios is not None:
                selected=[case['inputs'][0]+r*(case['inputs'][1]-case['inputs'][0]) for r in ratios]
                axis=axis.tick_values(selected if kind=='duration' else [{'Timestamp':{'value':str(round(v*multiplier)),'unit':'Microseconds'}} for v in selected])
        plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis).build();owned.append(plot)
        wire=plot.to_json();assert json.loads(wire)['version']==19
        restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
        request=output.request(restored,temporal_options);owned.append(request)
        try:frame=request.prepare()
        except c.ChartError as error:
            assert 'error' in case['result'],(index,error)
            records.append({'kind':kind,'index':index,'error':error.code});continue
        owned.append(frame);assert 'error' not in case['result'],index
        ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom').get('minor_ticks',[])
        assert len(ticks)==len(case['result']['values']),(index,ticks,case)
        for i,tick in enumerate(ticks):
            expected=case['result']['values'][i]
            if kind=='duration':assert math.isclose(tick['value']['Number'],expected,rel_tol=0,abs_tol=1e-10)
            else:
                stamp=tick['value']['Timestamp'];assert stamp['unit']=='Microseconds'
                assert int(stamp['value'])==round(expected*multiplier),(tick,expected)
            assert math.isclose(tick['position']/100.,case['result']['positions'][i],rel_tol=0,abs_tol=1e-12)
        records.append({'kind':kind,'index':index,'minor_ticks':ticks})
    finally:
        for value in reversed(owned):value.dispose()
for unit,factor,next_unit in [('s',1,'Milliseconds'),('ms',1000,'Microseconds'),('us',1000000,'Nanoseconds'),('ns',1000000000,None)]:
    owned=[]
    try:
        origin=1700000000*factor
        data=c.Data.columns({'x':c.timestamps([origin,origin+1],unit,'UTC'),'y':[1.,2.]});owned.append(data)
        enum={'s':'Seconds','ms':'Milliseconds','us':'Microseconds','ns':'Nanoseconds'}[unit]
        axis=c.x_axis().scale(c.scale_utc()).range(0.,100.).expansion({'mult':[0.,0.],'add':[0.,0.]}).tick_values([{'Timestamp':{'value':str(v),'unit':enum}} for v in [origin,origin+1]]).tick_format({'GgplotTime':{'pattern':'%Y'}})
        plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis).build();owned.append(plot)
        wire=plot.to_json();restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
        request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
        ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom')['minor_ticks']
        assert [v['position'] for v in ticks]==[0.,50.,100.]
        if next_unit is None:assert ticks[1]['value'] is None
        else:
            value=ticks[1]['value']['Timestamp'];assert value['unit']==next_unit and int(value['value'])==origin*1000+500
        records.append({'kind':'precision','unit':unit,'minor_ticks':ticks})
    finally:
        for value in reversed(owned):value.dispose()
fixture=json.loads((ROOT/'fixtures/parity/ggplot2/minor-discrete.json').read_text())
for index,case in enumerate(fixture['cases']):
    for family in ['auto','band','point']:
        for reversed_range in [False,True]:
            owned=[]
            try:
                n=case['count'];data=c.Data.columns({'x':c.categorical(list('abc')[:n]),'y':c.column([1.]*n,kind='float64')});owned.append(data)
                axis=c.x_axis().range(100. if reversed_range else 0.,0. if reversed_range else 100.)
                if family!='auto':axis=axis.scale({'band':c.scale_band,'point':c.scale_point}[family]())
                expansion={'none':{'mult':[0.,0.],'add':[0.,0.]},'wide':{'mult':[.2,.5],'add':[1.,2.]}}.get(case['expansion'])
                if expansion is not None:axis=axis.expansion(expansion)
                policy={'auto':'Automatic','hidden':'Hidden','empty':{'Numeric':[]},'explicit':{'Numeric':[{'number':'-Infinity'},-1,0,.5,1,1.5,2.5,3.5,4,{'number':'Infinity'},{'number':'NaN'}]}}[case['minor']]
                plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(axis.minor_breaks(policy)).build();owned.append(plot)
                wire=plot.to_json();assert json.loads(wire)['version']==19
                restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
                request=output.request(restored,options);owned.append(request);frame=request.prepare();owned.append(frame)
                ticks=next(g for g in frame.guides()['guides'] if g['spec']['side']=='Bottom').get('minor_ticks',[])
                assert len(ticks)==len(case['result']['values']),(index,family,ticks)
                for i,tick in enumerate(ticks):
                    assert tick['value']['Number']==case['result']['values'][i]
                    position=case['result']['positions'][i];position=1-position if reversed_range else position
                    assert math.isclose(tick['position']/100.,position,rel_tol=0,abs_tol=1e-12)
                records.append({'kind':'discrete','index':index,'family':family,'reversed':reversed_range,'minor_ticks':ticks})
            finally:
                for value in reversed(owned):value.dispose()
for index,values in enumerate([[{'Number':0.}],[{'Timestamp':{'value':'0','unit':'Milliseconds'}}],[None]*257]):
    owned=[]
    try:
        data=c.Data.columns({'x':c.timestamps([0,86400],'s','UTC'),'y':[1.,2.]});owned.append(data)
        plot=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(c.x_axis().scale(c.scale_utc()).minor_breaks({'Timestamps':values})).build();owned.append(plot)
        request=output.request(plot,options);owned.append(request)
        try:frame=request.prepare()
        except c.ChartError as error:records.append({'kind':'timestamp_rejection','index':index,'error':error.code})
        else:owned.append(frame);raise AssertionError('invalid timestamp minors accepted')
    finally:
        for value in reversed(owned):value.dispose()
output.dispose();assert len(records)==761
(out/'records.json').write_text(json.dumps(records,indent=2))
print('PASS Python: 448 numeric, 90 temporal, 216 discrete, four exact-origin and three invalid-input minor cases.')
